//! The built-in logic gates.
//!
//! A gate is an instance whose `module_ref` is the name of a built-in
//! module. The library of each design has these modules, so the canvas, the
//! nets and the validation handle a gate as they handle each instance. The
//! code generators are the exception: they write an assignment for a gate,
//! not an instance of a module.
//!
//! The name of a built-in module starts with `$`. No HDL identifier can
//! start with it, so a module from a source file never has such a name.

use std::path::PathBuf;

use crate::types::*;

/// The operation of a gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    And,
    Or,
    Xor,
    Not,
    Nand,
    Nor,
    Xnor,
}

pub const OPS: [Op; 7] = [
    Op::And,
    Op::Or,
    Op::Xor,
    Op::Not,
    Op::Nand,
    Op::Nor,
    Op::Xnor,
];

/// The first character of the name of each built-in module.
pub const PREFIX: char = '$';
/// The end of the name of the vector variant of a gate.
const VECTOR_SUFFIX: &str = "_vec";
/// The generic of a vector gate that gives the number of bits.
pub const WIDTH: &str = "WIDTH";
/// The output port of each gate.
pub const OUTPUT: &str = "y";

impl Op {
    /// The VHDL operator. It is also the name of the gate.
    fn word(self) -> &'static str {
        match self {
            Op::And => "and",
            Op::Or => "or",
            Op::Xor => "xor",
            Op::Not => "not",
            Op::Nand => "nand",
            Op::Nor => "nor",
            Op::Xnor => "xnor",
        }
    }

    /// The input ports, in the order of the operands.
    pub fn inputs(self) -> &'static [&'static str] {
        match self {
            Op::Not => &["a"],
            _ => &["a", "b"],
        }
    }
}

/// A built-in gate: the operation, and the variant. The scalar variant has
/// ports of one bit. The vector variant does the operation on each bit of
/// vectors of `WIDTH` bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gate {
    pub op: Op,
    pub vector: bool,
}

impl Gate {
    /// The gate that `module_ref` names. `None` for each other module.
    pub fn of(module_ref: &str) -> Option<Gate> {
        let name = module_ref.strip_prefix(PREFIX)?;
        let (word, vector) = match name.strip_suffix(VECTOR_SUFFIX) {
            Some(word) => (word, true),
            None => (name, false),
        };
        let op = OPS.into_iter().find(|op| op.word() == word)?;
        Some(Gate { op, vector })
    }

    /// The name of the built-in module of this gate.
    pub fn module_name(&self) -> String {
        let suffix = if self.vector { VECTOR_SUFFIX } else { "" };
        format!("{PREFIX}{}{suffix}", self.op.word())
    }

    /// The text of this gate in a menu of the window. The scalar gates and
    /// the vector gates have a menu each, so the text has the operation
    /// only.
    pub fn label(&self) -> String {
        self.op.word().to_uppercase()
    }

    /// The built-in module of this gate.
    pub fn module(&self) -> ModuleDef {
        let port_type = if self.vector {
            PortType::StdLogicVector(Range {
                high: RangeExpr::Expr(format!("{WIDTH}-1")),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            })
        } else {
            PortType::StdLogic
        };
        let port = |name: &str, direction: Direction| PortDef {
            name: name.to_string(),
            direction,
            port_type: port_type.clone(),
            bundle: None,
        };
        let mut ports: Vec<PortDef> = self
            .op
            .inputs()
            .iter()
            .map(|name| port(name, Direction::In))
            .collect();
        ports.push(port(OUTPUT, Direction::Out));
        let generics = if self.vector {
            vec![GenericDef {
                name: WIDTH.to_string(),
                type_name: "integer".to_string(),
                default_value: Some("8".to_string()),
            }]
        } else {
            Vec::new()
        };
        ModuleDef {
            name: self.module_name(),
            generics,
            ports,
            // A built-in module has no source file.
            source_path: PathBuf::new(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    /// The VHDL expression of the gate. `inputs` are the expressions of the
    /// input ports, in the order of `Op::inputs`.
    pub fn vhdl(&self, inputs: &[String]) -> String {
        match (self.op, inputs) {
            (Op::Not, [a]) => format!("not {a}"),
            (op, [a, b]) => format!("{a} {} {b}", op.word()),
            (op, _) => panic!("{op:?} with {} inputs", inputs.len()),
        }
    }

    /// The SystemVerilog expression of the gate. See `vhdl`.
    pub fn sv(&self, inputs: &[String]) -> String {
        match (self.op, inputs) {
            (Op::Not, [a]) => format!("~{a}"),
            (Op::And, [a, b]) => format!("{a} & {b}"),
            (Op::Or, [a, b]) => format!("{a} | {b}"),
            (Op::Xor, [a, b]) => format!("{a} ^ {b}"),
            (Op::Nand, [a, b]) => format!("~({a} & {b})"),
            (Op::Nor, [a, b]) => format!("~({a} | {b})"),
            (Op::Xnor, [a, b]) => format!("~({a} ^ {b})"),
            (op, _) => panic!("{op:?} with {} inputs", inputs.len()),
        }
    }
}

/// Each built-in module: the scalar gates, then the vector gates.
pub fn modules() -> Vec<ModuleDef> {
    [false, true]
        .into_iter()
        .flat_map(|vector| OPS.into_iter().map(move |op| Gate { op, vector }.module()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_built_in_module_names_its_gate() {
        let modules = modules();
        assert_eq!(modules.len(), 14);
        for module in &modules {
            let gate = Gate::of(&module.name).expect("a gate");
            assert_eq!(gate.module_name(), module.name);
            assert_eq!(
                module.ports.len(),
                gate.op.inputs().len() + 1,
                "{}",
                module.name
            );
            assert_eq!(module.generics.len(), usize::from(gate.vector));
        }
        let mut names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 14, "each name one time");
    }

    #[test]
    fn a_module_of_a_source_file_is_not_a_gate() {
        for name in ["and", "and_vec", "$", "$and_", "$mux", "counter", ""] {
            assert_eq!(Gate::of(name), None, "{name}");
        }
    }

    #[test]
    fn the_width_generic_gives_the_width_of_a_vector_gate() {
        let module = Gate {
            op: Op::Xor,
            vector: true,
        }
        .module();
        let generic_map = [(WIDTH.to_string(), "4".to_string())].into();
        let resolved = crate::codegen::resolve_port_type(
            &module.ports[0].port_type,
            &module.generics,
            &generic_map,
        );
        assert_eq!(
            resolved,
            PortType::StdLogicVector(Range {
                high: RangeExpr::Literal(3),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            })
        );
    }
}
