use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RangeExpr {
    Literal(i64),
    Expr(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RangeDir {
    Downto,
    To,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Range {
    pub high: RangeExpr,
    pub low: RangeExpr,
    pub dir: RangeDir,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortType {
    StdLogic,
    StdLogicVector(Range),
    Record(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    pub direction: Direction,
    pub port_type: PortType,
    pub bundle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericDef {
    pub name: String,
    pub type_name: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    pub name: String,
    pub generics: Vec<GenericDef>,
    pub ports: Vec<PortDef>,
    pub source_path: PathBuf,
    pub source_hash: u64,
    /// Unique module/component names referenced in this module's body.
    /// Not persisted — always re-derived by the parser.
    #[serde(default, skip_serializing)]
    pub dependencies: Vec<String>,
}

// --- Design-level types ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Vhdl,
    SystemVerilog,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SliceExpr {
    Bit(i32),
    Range { high: i32, low: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetRef {
    TopPort(String),
    InstancePort(String, String),
    TopPortSlice(String, SliceExpr),
    InstancePortSlice(String, String, SliceExpr),
    /// Literal tie, emitted verbatim in the target language
    /// (VHDL `'0'`, `"0101"`, `x"AB"`; SV `1'b0`, `8'hFF`).
    Constant(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub module_ref: String,
    pub generic_map: HashMap<String, String>,
    pub port_map: HashMap<String, Option<NetRef>>,
    pub position: (f32, f32),
    /// User-authored bundle groupings that override (or add to) the module's
    /// auto-detected bundles. Keyed by bundle name → ordered list of port
    /// names that belong to it. Empty for instances that don't use manual
    /// grouping; absent from v2 `.hdlc` files and deserializes as default.
    #[serde(default)]
    pub manual_bundles: HashMap<String, Vec<String>>,
    /// Optional slice of THIS instance's own port that's connected to the
    /// driver. The driver-side slice (if any) is encoded in the
    /// `port_map` value's `NetRef::*Slice` variant — these two slices are
    /// independent: a 32-bit `bus[7:0]` consumer slice can ride a 32-bit
    /// driver, or be sliced again from a wider net. Keyed by port name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub consumer_slices: HashMap<String, SliceExpr>,
    /// Set when a library re-parse dropped a port_map entry that referenced
    /// a now-missing port. Tells the user "review this instance". Clears on
    /// user acknowledgement or when the missing ports reappear.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dirty: bool,
}

/// Net identity key — used as the key in the alias map.
/// A net is identified by its driver (always the non-sliced form).
pub type NetId = NetRef;

impl SliceExpr {
    pub fn to_suffix(&self) -> String {
        match self {
            SliceExpr::Bit(i) => format!("[{i}]"),
            SliceExpr::Range { high, low } => format!("[{high}:{low}]"),
        }
    }

    fn parse_suffix(s: &str) -> Option<Self> {
        let inner = s.strip_prefix('[')?.strip_suffix(']')?;
        if let Some((h, l)) = inner.split_once(':') {
            let high = h.trim().parse().ok()?;
            let low = l.trim().parse().ok()?;
            Some(SliceExpr::Range { high, low })
        } else {
            Some(SliceExpr::Bit(inner.trim().parse().ok()?))
        }
    }
}

impl NetRef {
    /// Strip any slice to get the underlying driver net.
    pub fn base(&self) -> NetRef {
        match self {
            NetRef::TopPort(n) => NetRef::TopPort(n.clone()),
            NetRef::InstancePort(i, p) => NetRef::InstancePort(i.clone(), p.clone()),
            NetRef::TopPortSlice(n, _) => NetRef::TopPort(n.clone()),
            NetRef::InstancePortSlice(i, p, _) => NetRef::InstancePort(i.clone(), p.clone()),
            NetRef::Constant(v) => NetRef::Constant(v.clone()),
        }
    }

    /// Name of the instance this ref targets (if any).
    pub fn instance_name(&self) -> Option<&str> {
        match self {
            NetRef::InstancePort(i, _) | NetRef::InstancePortSlice(i, _, _) => Some(i.as_str()),
            _ => None,
        }
    }

    /// Serialize to a string key for use in JSON maps.
    pub fn to_key(&self) -> String {
        match self {
            NetRef::TopPort(name) => format!("top:{name}"),
            NetRef::InstancePort(inst, port) => format!("{inst}.{port}"),
            NetRef::TopPortSlice(name, slice) => format!("top:{name}{}", slice.to_suffix()),
            NetRef::InstancePortSlice(inst, port, slice) => {
                format!("{inst}.{port}{}", slice.to_suffix())
            }
            NetRef::Constant(v) => format!("const:{v}"),
        }
    }

    /// Deserialize from a string key.
    pub fn from_key(key: &str) -> Option<Self> {
        if let Some(v) = key.strip_prefix("const:") {
            return Some(NetRef::Constant(v.to_string()));
        }
        let (body, slice) = match key.find('[') {
            Some(i) => (&key[..i], SliceExpr::parse_suffix(&key[i..])),
            None => (key, None),
        };
        if let Some(name) = body.strip_prefix("top:") {
            Some(match slice {
                Some(s) => NetRef::TopPortSlice(name.to_string(), s),
                None => NetRef::TopPort(name.to_string()),
            })
        } else if let Some((inst, port)) = body.split_once('.') {
            Some(match slice {
                Some(s) => NetRef::InstancePortSlice(inst.to_string(), port.to_string(), s),
                None => NetRef::InstancePort(inst.to_string(), port.to_string()),
            })
        } else {
            None
        }
    }
}

fn serialize_aliases<S>(aliases: &HashMap<NetId, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(aliases.len()))?;
    for (k, v) in aliases {
        map.serialize_entry(&k.to_key(), v)?;
    }
    map.end()
}

fn deserialize_aliases<'de, D>(deserializer: D) -> Result<HashMap<NetId, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for (k, v) in string_map {
        if let Some(net_ref) = NetRef::from_key(&k) {
            result.insert(net_ref, v);
        }
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schematic {
    pub top_name: String,
    pub language: Language,
    pub top_generics: Vec<GenericDef>,
    pub top_ports: Vec<PortDef>,
    pub instances: Vec<Instance>,
    #[serde(
        serialize_with = "serialize_aliases",
        deserialize_with = "deserialize_aliases"
    )]
    pub aliases: HashMap<NetId, String>,
    pub library_paths: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_net_ref(nr: NetRef) {
        let json = serde_json::to_string(&nr).unwrap();
        let back: NetRef = serde_json::from_str(&json).unwrap();
        assert_eq!(nr, back, "serde roundtrip failed for {nr:?}");

        let key = nr.to_key();
        let parsed = NetRef::from_key(&key).unwrap();
        assert_eq!(nr, parsed, "to_key/from_key roundtrip failed for {nr:?}");
    }

    #[test]
    fn netref_serde_roundtrip_all_variants() {
        roundtrip_net_ref(NetRef::TopPort("clk".into()));
        roundtrip_net_ref(NetRef::InstancePort("u_a".into(), "dout".into()));
        roundtrip_net_ref(NetRef::TopPortSlice("bus".into(), SliceExpr::Bit(3)));
        roundtrip_net_ref(NetRef::TopPortSlice(
            "bus".into(),
            SliceExpr::Range { high: 7, low: 4 },
        ));
        roundtrip_net_ref(NetRef::InstancePortSlice(
            "u_cnt".into(),
            "count".into(),
            SliceExpr::Bit(0),
        ));
        roundtrip_net_ref(NetRef::InstancePortSlice(
            "u_cnt".into(),
            "count".into(),
            SliceExpr::Range { high: 7, low: 4 },
        ));
        roundtrip_net_ref(NetRef::Constant("'0'".into()));
        roundtrip_net_ref(NetRef::Constant("x\"AB\"".into()));
        roundtrip_net_ref(NetRef::Constant("8'hFF".into()));
    }

    #[test]
    fn netref_base_strips_slice() {
        let nr = NetRef::InstancePortSlice(
            "u_a".into(),
            "dout".into(),
            SliceExpr::Range { high: 7, low: 4 },
        );
        assert_eq!(
            nr.base(),
            NetRef::InstancePort("u_a".into(), "dout".into())
        );
        assert_eq!(
            NetRef::TopPortSlice("bus".into(), SliceExpr::Bit(3)).base(),
            NetRef::TopPort("bus".into())
        );
    }

    #[test]
    fn netref_instance_name_accessor() {
        assert_eq!(
            NetRef::InstancePort("u_a".into(), "p".into()).instance_name(),
            Some("u_a")
        );
        assert_eq!(
            NetRef::InstancePortSlice("u_b".into(), "p".into(), SliceExpr::Bit(0)).instance_name(),
            Some("u_b")
        );
        assert_eq!(NetRef::TopPort("clk".into()).instance_name(), None);
        assert_eq!(
            NetRef::TopPortSlice("bus".into(), SliceExpr::Bit(0)).instance_name(),
            None
        );
    }
}
