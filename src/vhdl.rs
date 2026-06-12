use std::path::Path;

use std::collections::HashMap;

use vhdl_lang::ast::{
    AnyDesignUnit, AnyPrimaryUnit, AnySecondaryUnit, ArchitectureBody, ConcurrentStatement,
    Designator, InstantiatedUnit, InterfaceDeclaration, LabeledConcurrentStatement, ModeIndication,
    Name,
};
use vhdl_lang::{Diagnostic, VHDLParser, VHDLStandard};

use crate::ParseError;
use crate::types::{
    Direction, GenericDef, ModuleDef, PortDef, PortType, Range, RangeDir, RangeExpr,
};

pub fn parse_vhdl(path: &Path) -> Result<Vec<ModuleDef>, ParseError> {
    let source_bytes = std::fs::read(path)?;
    let source_hash = seahash::hash(&source_bytes);

    let parser = VHDLParser::new(VHDLStandard::VHDL2008);
    // Collect parser diagnostics instead of discarding them. vhdl_lang
    // recovers from syntax errors and returns Ok with whatever it salvaged,
    // so a garbage file used to load silently as an empty library.
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let (_source, design_file) = parser
        .parse_design_file(path, &mut diagnostics)
        .map_err(|e| ParseError::VhdlParse(e.to_string()))?;

    let mut modules = Vec::new();
    // entity_name -> dependency set
    let mut deps_by_entity: HashMap<String, Vec<String>> = HashMap::new();

    // First pass: walk architectures to collect dependencies per associated entity.
    for (_tokens, unit) in &design_file.design_units {
        if let AnyDesignUnit::Secondary(AnySecondaryUnit::Architecture(arch)) = unit {
            let entity_name = arch.entity_name.item.item.to_string();
            let deps = deps_by_entity.entry(entity_name).or_default();
            collect_dependencies(arch, deps);
        }
    }

    for (_tokens, unit) in &design_file.design_units {
        if let AnyDesignUnit::Primary(AnyPrimaryUnit::Entity(entity)) = unit {
            let name = entity.ident.tree.item.to_string();

            let generics = extract_generics(entity.generic_clause.as_ref());
            let mut ports = extract_ports(entity.port_clause.as_ref());

            // Initialize bundle to None for all ports (bundle detection is a separate pass)
            for port in &mut ports {
                port.bundle = None;
            }

            let dependencies = deps_by_entity.remove(&name).unwrap_or_default();

            modules.push(ModuleDef {
                name,
                generics,
                ports,
                source_path: path.to_path_buf(),
                source_hash,
                dependencies,
            });
        }
    }

    // No entities AND syntax diagnostics → the file is broken, not merely
    // entity-free (packages-only files parse with no diagnostics). Surface
    // it instead of silently loading an empty library.
    if modules.is_empty()
        && let Some(d) = diagnostics.first()
    {
        let line = d.pos.range.start.line + 1; // 0-based in vhdl_lang
        return Err(ParseError::VhdlParse(format!("line {line}: {}", d.message)));
    }

    Ok(modules)
}

/// Collect unique component/entity names referenced in an architecture body.
fn collect_dependencies(arch: &ArchitectureBody, deps: &mut Vec<String>) {
    collect_statements(&arch.statements, deps);
}

fn collect_statements(stmts: &[LabeledConcurrentStatement], deps: &mut Vec<String>) {
    for labeled in stmts {
        collect_concurrent(&labeled.statement.item, deps);
    }
}

fn collect_concurrent(stmt: &ConcurrentStatement, deps: &mut Vec<String>) {
    match stmt {
        ConcurrentStatement::Instance(inst) => {
            if let Some(n) = name_suffix(match &inst.unit {
                InstantiatedUnit::Component(n) => &n.item,
                InstantiatedUnit::Entity(n, _) => &n.item,
                InstantiatedUnit::Configuration(n) => &n.item,
            }) {
                push_unique(deps, n);
            }
        }
        ConcurrentStatement::ForGenerate(g) => {
            collect_statements(&g.body.statements, deps);
        }
        ConcurrentStatement::IfGenerate(g) => {
            for cond in &g.conds.conditionals {
                collect_statements(&cond.item.statements, deps);
            }
            if let Some((else_body, _)) = &g.conds.else_item {
                collect_statements(&else_body.statements, deps);
            }
        }
        ConcurrentStatement::CaseGenerate(g) => {
            for alt in &g.sels.alternatives {
                collect_statements(&alt.item.statements, deps);
            }
        }
        ConcurrentStatement::Block(block) => {
            collect_statements(&block.statements, deps);
        }
        _ => {}
    }
}

/// Extract the trailing identifier from a VHDL name (ignoring selection prefix).
/// `flipflop` -> "flipflop"; `work.flipflop` -> "flipflop".
fn name_suffix(name: &Name) -> Option<String> {
    match name {
        Name::Designator(d) => match &d.item {
            Designator::Identifier(sym) => Some(sym.name_utf8()),
            _ => None,
        },
        Name::Selected(_, suffix) => match &suffix.item.item {
            Designator::Identifier(sym) => Some(sym.name_utf8()),
            _ => None,
        },
        _ => None,
    }
}

fn push_unique(deps: &mut Vec<String>, name: String) {
    if !deps.iter().any(|n| n == &name) {
        deps.push(name);
    }
}

fn extract_generics(generic_clause: Option<&vhdl_lang::ast::InterfaceList>) -> Vec<GenericDef> {
    let Some(list) = generic_clause else {
        return Vec::new();
    };

    let mut generics = Vec::new();
    for item in &list.items {
        if let InterfaceDeclaration::Object(obj) = item
            && let ModeIndication::Simple(smi) = &obj.mode
        {
            let type_name = format!("{}", smi.subtype_indication.type_mark.item);
            let default_value = smi.expression.as_ref().map(|e| format!("{}", e.item));

            for ident in &obj.idents {
                generics.push(GenericDef {
                    name: ident.tree.item.to_string(),
                    type_name: type_name.clone(),
                    default_value: default_value.clone(),
                });
            }
        }
    }

    generics
}

fn extract_ports(port_clause: Option<&vhdl_lang::ast::InterfaceList>) -> Vec<PortDef> {
    let Some(list) = port_clause else {
        return Vec::new();
    };

    let mut ports = Vec::new();
    for item in &list.items {
        if let InterfaceDeclaration::Object(obj) = item
            && let ModeIndication::Simple(smi) = &obj.mode
        {
            let direction = match smi.mode.as_ref().map(|m| m.item) {
                Some(vhdl_lang::ast::Mode::In) | None => Direction::In,
                Some(vhdl_lang::ast::Mode::Out) => Direction::Out,
                Some(vhdl_lang::ast::Mode::InOut) => Direction::InOut,
                Some(vhdl_lang::ast::Mode::Buffer) => Direction::Out,
                Some(vhdl_lang::ast::Mode::Linkage) => Direction::InOut,
            };

            let port_type = map_vhdl_type(&smi.subtype_indication);

            for ident in &obj.idents {
                ports.push(PortDef {
                    name: ident.tree.item.to_string(),
                    direction: direction.clone(),
                    port_type: port_type.clone(),
                    bundle: None,
                });
            }
        }
    }

    ports
}

fn map_vhdl_type(subtype: &vhdl_lang::ast::SubtypeIndication) -> PortType {
    let type_name = name_to_lowercase(&subtype.type_mark.item);

    match type_name.as_str() {
        "std_logic" | "std_ulogic" => PortType::StdLogic,
        "std_logic_vector" | "std_ulogic_vector" => {
            if let Some(range) = extract_range_from_subtype(subtype) {
                PortType::StdLogicVector(range)
            } else {
                PortType::Other(format!("{}", subtype))
            }
        }
        "signed" | "unsigned" => {
            if let Some(range) = extract_range_from_subtype(subtype) {
                PortType::StdLogicVector(range)
            } else {
                PortType::Other(format!("{}", subtype))
            }
        }
        _ => {
            // Check if it might be a record type (no constraint, not a known scalar)
            if is_likely_record(&type_name) {
                PortType::Record(type_name)
            } else {
                PortType::Other(format!("{}", subtype))
            }
        }
    }
}

fn name_to_lowercase(name: &Name) -> String {
    match name {
        Name::Designator(with_ref) => match &with_ref.item {
            Designator::Identifier(sym) => sym.to_string().to_lowercase(),
            _ => format!("{name}").to_lowercase(),
        },
        _ => format!("{name}").to_lowercase(),
    }
}

fn extract_range_from_subtype(subtype: &vhdl_lang::ast::SubtypeIndication) -> Option<Range> {
    use vhdl_lang::ast::{DiscreteRange, SubtypeConstraint};

    let constraint = subtype.constraint.as_ref()?;

    match &constraint.item {
        SubtypeConstraint::Array(ranges, _) => {
            let first = ranges.first()?;
            match &first.item {
                DiscreteRange::Range(range) => extract_range(range),
                DiscreteRange::Discrete(_, Some(range)) => extract_range(range),
                _ => None,
            }
        }
        SubtypeConstraint::Range(range) => extract_range(range),
        _ => None,
    }
}

fn extract_range(range: &vhdl_lang::ast::Range) -> Option<Range> {
    use vhdl_lang::ast::Range as VhdlRange;

    match range {
        VhdlRange::Range(rc) => {
            let high = expr_to_range_expr(&rc.left_expr.item);
            let low = expr_to_range_expr(&rc.right_expr.item);
            let dir = match rc.direction {
                vhdl_lang::ast::Direction::Descending => RangeDir::Downto,
                vhdl_lang::ast::Direction::Ascending => RangeDir::To,
            };

            // For "downto", left is high, right is low
            // For "to", left is low, right is high
            match dir {
                RangeDir::Downto => Some(Range { high, low, dir }),
                RangeDir::To => Some(Range {
                    high: low,
                    low: high,
                    dir,
                }),
            }
        }
        VhdlRange::Attribute(_) => None,
    }
}

fn expr_to_range_expr(expr: &vhdl_lang::ast::Expression) -> RangeExpr {
    use vhdl_lang::ast::Expression;

    match expr {
        Expression::Literal(lit) => {
            let s = format!("{lit}");
            if let Ok(n) = s.parse::<i64>() {
                RangeExpr::Literal(n)
            } else {
                RangeExpr::Expr(s)
            }
        }
        _ => RangeExpr::Expr(format!("{expr}")),
    }
}

fn is_likely_record(type_name: &str) -> bool {
    // Heuristic: types ending in _t, _type, _record, or containing "rec" are likely records.
    // Types like "integer", "natural", "boolean", "real" are not.
    let known_scalars = [
        "integer",
        "natural",
        "positive",
        "boolean",
        "bit",
        "bit_vector",
        "real",
        "time",
        "character",
        "string",
    ];
    if known_scalars.contains(&type_name) {
        return false;
    }
    type_name.ends_with("_t")
        || type_name.ends_with("_type")
        || type_name.ends_with("_record")
        || type_name.contains("rec")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_vhdl_str(content: &str) -> Result<Vec<ModuleDef>, ParseError> {
        let mut f = NamedTempFile::with_suffix(".vhd").unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        parse_vhdl(f.path())
    }

    #[test]
    fn simple_entity() {
        let vhdl = r#"
library ieee;
use ieee.std_logic_1164.all;

entity counter is
  port (
    clk   : in  std_logic;
    rst_n : in  std_logic;
    count : out std_logic_vector(7 downto 0)
  );
end entity counter;
"#;
        let modules = parse_vhdl_str(vhdl).unwrap();
        assert_eq!(modules.len(), 1);
        let m = &modules[0];
        assert_eq!(m.name, "counter");
        assert_eq!(m.ports.len(), 3);

        assert_eq!(m.ports[0].name, "clk");
        assert_eq!(m.ports[0].direction, Direction::In);
        assert!(matches!(m.ports[0].port_type, PortType::StdLogic));

        assert_eq!(m.ports[2].name, "count");
        assert_eq!(m.ports[2].direction, Direction::Out);
        if let PortType::StdLogicVector(ref range) = m.ports[2].port_type {
            assert_eq!(range.high, RangeExpr::Literal(7));
            assert_eq!(range.low, RangeExpr::Literal(0));
        } else {
            panic!("expected StdLogicVector");
        }
    }

    #[test]
    fn entity_with_generics() {
        let vhdl = r#"
library ieee;
use ieee.std_logic_1164.all;

entity fifo_sync is
  generic (
    DEPTH : integer := 256;
    WIDTH : integer := 8
  );
  port (
    clk : in std_logic
  );
end entity fifo_sync;
"#;
        let modules = parse_vhdl_str(vhdl).unwrap();
        assert_eq!(modules.len(), 1);
        let m = &modules[0];
        assert_eq!(m.generics.len(), 2);
        assert_eq!(m.generics[0].name, "DEPTH");
        assert_eq!(m.generics[0].type_name, "integer");
        assert_eq!(m.generics[0].default_value, Some("256".to_string()));
        assert_eq!(m.generics[1].name, "WIDTH");
    }

    #[test]
    fn multiple_entities() {
        let vhdl = r#"
library ieee;
use ieee.std_logic_1164.all;

entity foo is
  port (a : in std_logic);
end entity foo;

entity bar is
  port (b : out std_logic);
end entity bar;
"#;
        let modules = parse_vhdl_str(vhdl).unwrap();
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].name, "foo");
        assert_eq!(modules[1].name, "bar");
    }

    #[test]
    fn empty_file() {
        let modules = parse_vhdl_str("").unwrap();
        assert!(modules.is_empty());
    }

    #[test]
    fn garbage_reports_parse_error() {
        // vhdl_lang recovers from syntax errors and returns Ok — without the
        // diagnostics check this loaded silently as an empty library.
        let result = parse_vhdl_str("this is not VHDL at all }{");
        assert!(result.is_err(), "garbage VHDL must surface a parse error");
    }

    #[test]
    fn parameterized_width() {
        let vhdl = r#"
library ieee;
use ieee.std_logic_1164.all;

entity param_width is
  generic (
    WIDTH : integer := 8
  );
  port (
    data : out std_logic_vector(WIDTH-1 downto 0)
  );
end entity param_width;
"#;
        let modules = parse_vhdl_str(vhdl).unwrap();
        let m = &modules[0];
        if let PortType::StdLogicVector(ref range) = m.ports[0].port_type {
            // Should be an expression, not a literal
            assert!(matches!(range.high, RangeExpr::Expr(_)));
            assert_eq!(range.low, RangeExpr::Literal(0));
        } else {
            panic!("expected StdLogicVector");
        }
    }

    #[test]
    fn inout_direction() {
        let vhdl = r#"
library ieee;
use ieee.std_logic_1164.all;

entity bidir is
  port (
    sda : inout std_logic
  );
end entity bidir;
"#;
        let modules = parse_vhdl_str(vhdl).unwrap();
        assert_eq!(modules[0].ports[0].direction, Direction::InOut);
    }

    #[test]
    fn hash_stability() {
        let vhdl = "entity x is end entity x;\n";
        let m1 = parse_vhdl_str(vhdl).unwrap();
        let m2 = parse_vhdl_str(vhdl).unwrap();
        assert_eq!(m1[0].source_hash, m2[0].source_hash);
    }
}
