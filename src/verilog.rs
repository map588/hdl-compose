use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sv_parser::{RefNode, SyntaxTree, parse_sv, unwrap_node};

use crate::ParseError;
use crate::types::{
    Direction, GenericDef, ModuleDef, PortDef, PortType, Range, RangeDir, RangeExpr,
};

pub fn parse_verilog(path: &Path) -> Result<Vec<ModuleDef>, ParseError> {
    let source_bytes = std::fs::read(path)?;
    let source_hash = seahash::hash(&source_bytes);

    let defines = HashMap::new();
    let includes: Vec<PathBuf> = Vec::new();

    let (tree, _) = parse_sv(path, &defines, &includes, false, false)
        .map_err(|e| ParseError::VerilogParse(format!("{e}")))?;

    // Collect all module instantiations in the file (flattened).
    // In single-module-per-file convention these are all dependencies of the one module.
    // In multi-module files this overapproximates; accepted for v1.
    let mut file_deps: Vec<String> = Vec::new();
    for node in &tree {
        if let RefNode::ModuleInstantiation(inst) = node {
            let id_node = RefNode::ModuleIdentifier(&inst.nodes.0);
            if let Some(name) = get_identifier(&tree, id_node)
                && !file_deps.iter().any(|d| d == &name)
            {
                file_deps.push(name);
            }
        }
    }

    let mut modules = Vec::new();

    for node in &tree {
        match node {
            RefNode::ModuleDeclarationAnsi(m) => {
                if let Some(mut module) = extract_ansi_module(&tree, m, path, source_hash) {
                    module.dependencies = file_deps.clone();
                    modules.push(module);
                }
            }
            RefNode::ModuleDeclarationNonansi(m) => {
                if let Some(mut module) = extract_nonansi_module(&tree, m, path, source_hash) {
                    module.dependencies = file_deps.clone();
                    modules.push(module);
                }
            }
            _ => {}
        }
    }

    Ok(modules)
}

fn get_identifier(tree: &SyntaxTree, node: RefNode) -> Option<String> {
    let locate = unwrap_node!(node, SimpleIdentifier, EscapedIdentifier)?;
    match locate {
        RefNode::SimpleIdentifier(id) => Some(tree.get_str(id)?.trim().to_string()),
        RefNode::EscapedIdentifier(id) => Some(tree.get_str(id)?.trim().to_string()),
        _ => None,
    }
}

fn extract_ansi_module(
    tree: &SyntaxTree,
    module: &sv_parser::ModuleDeclarationAnsi,
    path: &Path,
    source_hash: u64,
) -> Option<ModuleDef> {
    let header = &module.nodes.0;

    // Module name
    let name_node = unwrap_node!(RefNode::ModuleAnsiHeader(header), ModuleIdentifier)?;
    let name = get_identifier(tree, name_node)?;

    // Parameters
    let generics = extract_parameters(tree, RefNode::ModuleAnsiHeader(header));

    // Ports
    let ports = extract_ansi_ports(tree, RefNode::ModuleAnsiHeader(header));

    Some(ModuleDef {
        name,
        generics,
        ports,
        source_path: path.to_path_buf(),
        source_hash,
        dependencies: Vec::new(),
    })
}

fn extract_nonansi_module(
    tree: &SyntaxTree,
    module: &sv_parser::ModuleDeclarationNonansi,
    path: &Path,
    source_hash: u64,
) -> Option<ModuleDef> {
    let header = &module.nodes.0;

    // Module name
    let name_node = unwrap_node!(RefNode::ModuleNonansiHeader(header), ModuleIdentifier)?;
    let name = get_identifier(tree, name_node)?;

    // Parameters
    let generics = extract_parameters(tree, RefNode::ModuleNonansiHeader(header));

    // Ports — for non-ANSI, we need to look at the module body for port declarations
    let ports = extract_nonansi_ports(tree, module);

    Some(ModuleDef {
        name,
        generics,
        ports,
        source_path: path.to_path_buf(),
        source_hash,
        dependencies: Vec::new(),
    })
}

fn extract_parameters(tree: &SyntaxTree, header: RefNode) -> Vec<GenericDef> {
    let mut generics = Vec::new();

    for node in header {
        if let RefNode::ParameterDeclarationParam(param) = node {
            // Get data type as string
            let type_str = get_node_text(tree, RefNode::DataTypeOrImplicit(&param.nodes.1));

            // Get each parameter assignment
            for assign_node in RefNode::ListOfParamAssignments(&param.nodes.2) {
                if let RefNode::ParamAssignment(assign) = assign_node {
                    let param_name_node =
                        unwrap_node!(RefNode::ParamAssignment(assign), ParameterIdentifier);
                    if let Some(param_name) = param_name_node.and_then(|n| get_identifier(tree, n))
                    {
                        let default_value = assign.nodes.2.as_ref().map(|(_, expr)| {
                            get_node_text(tree, RefNode::ConstantParamExpression(expr))
                        });

                        generics.push(GenericDef {
                            name: param_name,
                            type_name: type_str.clone(),
                            default_value,
                        });
                    }
                }
            }
        }
    }

    generics
}

fn extract_ansi_ports(tree: &SyntaxTree, header: RefNode) -> Vec<PortDef> {
    let mut ports = Vec::new();
    let mut current_dir = Direction::In; // Default direction per spec

    for node in header {
        match node {
            RefNode::AnsiPortDeclarationNet(decl) => {
                let dir = extract_port_direction_from_net_header(
                    tree,
                    RefNode::AnsiPortDeclarationNet(decl),
                );
                if let Some(d) = dir {
                    current_dir = d;
                }

                let port_type = extract_port_type_from_ansi_net(tree, decl);
                let port_name_node =
                    unwrap_node!(RefNode::AnsiPortDeclarationNet(decl), PortIdentifier);
                if let Some(port_name) = port_name_node.and_then(|n| get_identifier(tree, n)) {
                    ports.push(PortDef {
                        name: port_name,
                        direction: current_dir.clone(),
                        port_type,
                        bundle: None,
                    });
                }
            }
            RefNode::AnsiPortDeclarationVariable(decl) => {
                let dir = extract_port_direction_from_var_header(
                    tree,
                    RefNode::AnsiPortDeclarationVariable(decl),
                );
                if let Some(d) = dir {
                    current_dir = d;
                }

                let port_type =
                    extract_port_type_from_node(tree, RefNode::AnsiPortDeclarationVariable(decl));
                let port_name_node =
                    unwrap_node!(RefNode::AnsiPortDeclarationVariable(decl), PortIdentifier);
                if let Some(port_name) = port_name_node.and_then(|n| get_identifier(tree, n)) {
                    ports.push(PortDef {
                        name: port_name,
                        direction: current_dir.clone(),
                        port_type,
                        bundle: None,
                    });
                }
            }
            _ => {}
        }
    }

    ports
}

fn extract_nonansi_ports(
    tree: &SyntaxTree,
    module: &sv_parser::ModuleDeclarationNonansi,
) -> Vec<PortDef> {
    let mut ports = Vec::new();

    // Collect port names from the header's port list for ordering
    let mut port_order: Vec<String> = Vec::new();
    for node in RefNode::ModuleNonansiHeader(&module.nodes.0) {
        if let RefNode::PortReference(pr) = node {
            let name_node = unwrap_node!(RefNode::PortReference(pr), PortIdentifier);
            if let Some(name) = name_node.and_then(|n| get_identifier(tree, n)) {
                port_order.push(name);
            }
        }
    }

    // Collect port declarations from the module body
    let mut port_map: HashMap<String, PortDef> = HashMap::new();

    for node in RefNode::ModuleDeclarationNonansi(module) {
        match node {
            RefNode::InputDeclarationNet(decl) => {
                collect_port_names(
                    tree,
                    RefNode::InputDeclarationNet(decl),
                    Direction::In,
                    &mut port_map,
                );
            }
            RefNode::InputDeclarationVariable(decl) => {
                collect_port_names(
                    tree,
                    RefNode::InputDeclarationVariable(decl),
                    Direction::In,
                    &mut port_map,
                );
            }
            RefNode::OutputDeclarationNet(decl) => {
                collect_port_names(
                    tree,
                    RefNode::OutputDeclarationNet(decl),
                    Direction::Out,
                    &mut port_map,
                );
            }
            RefNode::OutputDeclarationVariable(decl) => {
                collect_port_names(
                    tree,
                    RefNode::OutputDeclarationVariable(decl),
                    Direction::Out,
                    &mut port_map,
                );
            }
            RefNode::InoutDeclaration(decl) => {
                collect_port_names(
                    tree,
                    RefNode::InoutDeclaration(decl),
                    Direction::InOut,
                    &mut port_map,
                );
            }
            _ => {}
        }
    }

    // Return ports in declaration order
    for name in &port_order {
        if let Some(port) = port_map.remove(name) {
            ports.push(port);
        }
    }

    ports
}

fn collect_port_names(
    tree: &SyntaxTree,
    node: RefNode,
    direction: Direction,
    port_map: &mut HashMap<String, PortDef>,
) {
    let port_type = extract_port_type_from_node(tree, node.clone());

    for sub in node {
        if let RefNode::PortIdentifier(pi) = sub {
            let name_node = unwrap_node!(RefNode::PortIdentifier(pi), Identifier);
            if let Some(name) = name_node.and_then(|n| get_identifier(tree, n)) {
                port_map.insert(
                    name.clone(),
                    PortDef {
                        name,
                        direction: direction.clone(),
                        port_type: port_type.clone(),
                        bundle: None,
                    },
                );
            }
        }
    }
}

fn extract_port_direction_from_net_header(tree: &SyntaxTree, node: RefNode) -> Option<Direction> {
    for sub in node {
        if let RefNode::PortDirection(pd) = sub {
            return Some(parse_port_direction(tree, pd));
        }
    }
    None
}

fn extract_port_direction_from_var_header(tree: &SyntaxTree, node: RefNode) -> Option<Direction> {
    for sub in node {
        if let RefNode::PortDirection(pd) = sub {
            return Some(parse_port_direction(tree, pd));
        }
    }
    None
}

fn parse_port_direction(tree: &SyntaxTree, pd: &sv_parser::PortDirection) -> Direction {
    let text = get_node_text(tree, RefNode::PortDirection(pd)).to_lowercase();
    match text.trim() {
        "input" => Direction::In,
        "output" => Direction::Out,
        "inout" => Direction::InOut,
        _ => Direction::In,
    }
}

fn extract_port_type_from_ansi_net(
    tree: &SyntaxTree,
    decl: &sv_parser::AnsiPortDeclarationNet,
) -> PortType {
    extract_port_type_from_node(tree, RefNode::AnsiPortDeclarationNet(decl))
}

fn extract_port_type_from_node(tree: &SyntaxTree, node: RefNode) -> PortType {
    // Look for packed dimensions (width specifiers like [7:0])
    let mut has_range = false;
    let mut range_text = String::new();

    for sub in node.clone() {
        if let RefNode::PackedDimensionRange(r) = sub {
            has_range = true;
            // Extract the constant range text
            for inner in RefNode::PackedDimensionRange(r) {
                if let RefNode::ConstantRange(cr) = inner {
                    range_text = get_node_text(tree, RefNode::ConstantRange(cr));
                    break;
                }
            }
            break;
        }
    }

    if has_range && let Some(range) = parse_verilog_range(&range_text) {
        return PortType::StdLogicVector(range);
    }

    // Check if it's a single-bit signal (no packed dimensions)
    // Look for data type keywords
    let mut has_data_type = false;
    for sub in node {
        match sub {
            RefNode::IntegerVectorType(_) => {
                has_data_type = true;
                if !has_range {
                    return PortType::StdLogic;
                }
            }
            RefNode::DataType(_) => {
                has_data_type = true;
            }
            _ => {}
        }
    }

    if !has_data_type && !has_range {
        // Implicit single-bit wire
        return PortType::StdLogic;
    }

    PortType::StdLogic
}

fn parse_verilog_range(range_text: &str) -> Option<Range> {
    // Range text is like "7 : 0" or "WIDTH-1 : 0"
    let parts: Vec<&str> = range_text.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    let high = if let Ok(n) = left.parse::<i64>() {
        RangeExpr::Literal(n)
    } else {
        RangeExpr::Expr(left.to_string())
    };

    let low = if let Ok(n) = right.parse::<i64>() {
        RangeExpr::Literal(n)
    } else {
        RangeExpr::Expr(right.to_string())
    };

    Some(Range {
        high,
        low,
        dir: RangeDir::Downto, // Verilog [high:low] is always "downto" semantics
    })
}

fn get_node_text(tree: &SyntaxTree, node: RefNode) -> String {
    let mut text = String::new();
    for sub in node {
        if let RefNode::Locate(loc) = sub
            && let Some(s) = tree.get_str(loc)
        {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(s);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_verilog_str(content: &str, ext: &str) -> Result<Vec<ModuleDef>, ParseError> {
        let suffix = format!(".{ext}");
        let mut f = NamedTempFile::with_suffix(&suffix).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        parse_verilog(f.path())
    }

    #[test]
    fn simple_ansi_module() {
        let v = r#"
module counter (
    input wire clk,
    input wire rst_n,
    output reg [7:0] count
);
endmodule
"#;
        let modules = parse_verilog_str(v, "v").unwrap();
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
            panic!("expected StdLogicVector, got {:?}", m.ports[2].port_type);
        }
    }

    #[test]
    fn module_with_parameters() {
        let v = r#"
module fifo #(
    parameter WIDTH = 8,
    parameter DEPTH = 256
) (
    input wire clk
);
endmodule
"#;
        let modules = parse_verilog_str(v, "v").unwrap();
        let m = &modules[0];
        assert_eq!(m.generics.len(), 2);
        assert_eq!(m.generics[0].name, "WIDTH");
        assert_eq!(m.generics[0].default_value, Some("8".to_string()));
        assert_eq!(m.generics[1].name, "DEPTH");
    }

    #[test]
    fn nonansi_ports() {
        let v = r#"
module foo(a, b);
    input a;
    output b;
endmodule
"#;
        let modules = parse_verilog_str(v, "v").unwrap();
        let m = &modules[0];
        assert_eq!(m.ports.len(), 2);
        assert_eq!(m.ports[0].name, "a");
        assert_eq!(m.ports[0].direction, Direction::In);
        assert_eq!(m.ports[1].name, "b");
        assert_eq!(m.ports[1].direction, Direction::Out);
    }

    #[test]
    fn sv_extension() {
        let v = r#"
module sv_mod (
    input logic clk,
    output logic [15:0] data
);
endmodule
"#;
        let modules = parse_verilog_str(v, "sv").unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "sv_mod");
    }

    #[test]
    fn inout_direction() {
        let v = r#"
module bidir (
    inout wire sda
);
endmodule
"#;
        let modules = parse_verilog_str(v, "v").unwrap();
        assert_eq!(modules[0].ports[0].direction, Direction::InOut);
    }

    #[test]
    fn empty_file() {
        // sv-parser may error on truly empty file, but that's OK
        let result = parse_verilog_str("", "v");
        // Either empty vec or error is acceptable for empty file
        match result {
            Ok(modules) => assert!(modules.is_empty()),
            Err(_) => {} // parse error on empty file is acceptable
        }
    }

    #[test]
    fn parameterized_width() {
        let v = r#"
module param_mod #(
    parameter WIDTH = 8
) (
    input wire [WIDTH-1:0] data
);
endmodule
"#;
        let modules = parse_verilog_str(v, "v").unwrap();
        let m = &modules[0];
        if let PortType::StdLogicVector(ref range) = m.ports[0].port_type {
            assert!(matches!(range.high, RangeExpr::Expr(_)));
            assert_eq!(range.low, RangeExpr::Literal(0));
        } else {
            panic!("expected StdLogicVector");
        }
    }
}
