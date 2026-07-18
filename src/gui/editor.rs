//! Per-instance mini-editor buffer grammar: render an instance's
//! generic/port bindings as an editable text buffer, parse it back, and
//! detect completion contexts. Pure functions over the model — the
//! QPlainTextEdit mechanics stay in C++ (MainWindow), and the top-level
//! entity buffer has its own grammar in bridge.rs.
//!
//! Ported from the hand-rolled C++ parser that lived in
//! `src/gui/editor_buffer.cpp`, with one fix: slice RHS values render in
//! the same form the parser accepts (`u_a.dout[3]`, `bus[7:0]`) — the old
//! renderer emitted `top:bus[0]` for top-port slices, which its own
//! validator rejected, leaving the instance uncommittable.

use crate::types::{Language, ModuleDef, NetRef, Schematic};

/// Result of parsing an editor buffer.
#[derive(Debug, Default)]
pub struct ParsedBuffer {
    /// (generic name, value expression)
    pub generic_commits: Vec<(String, String)>,
    /// (lhs incl. optional consumer-slice suffix, cleaned rhs; empty = open)
    pub port_commits: Vec<(String, String)>,
    /// (0-based line index, message)
    pub errors: Vec<(usize, String)>,
}

/// Completion context at the cursor. kind: 0 = none (hide popup),
/// 1 = RHS (offer all drivers), 2 = dot-port (offer `instance`'s ports).
#[derive(Debug, PartialEq)]
pub struct CompletionCtx {
    pub kind: i32,
    pub prefix: String,
    pub instance: String,
}

/// RHS in the form the parser accepts (and `parse_net_rhs` re-parses).
fn rhs_editor_form(net: &NetRef) -> String {
    match net {
        NetRef::TopPort(n) => n.clone(),
        NetRef::InstancePort(i, p) => format!("{i}.{p}"),
        NetRef::TopPortSlice(n, sl) => format!("{n}{}", sl.to_suffix()),
        NetRef::InstancePortSlice(i, p, sl) => format!("{i}.{p}{}", sl.to_suffix()),
        NetRef::Constant(lit) => lit.clone(),
    }
}

/// Render one instance's bindings as a component-instantiation buffer.
/// Empty string if `instance_name` is empty or unknown.
///
/// VHDL form:
/// ```text
/// u_fifo : fifo_sync
///   generic map (
///     WIDTH => 16,
///   )
///   port map (
///     clk => clk_sys,
///   );
/// ```
///
/// SV form: `fifo_sync #( .WIDTH(16), ) u_fifo ( .clk(clk_sys), );`
///
/// Trailing comma on every entry — punctuation is uniform; the parser
/// strips them.
pub fn render_instance_buffer(s: &Schematic, library: &[ModuleDef], instance_name: &str) -> String {
    if instance_name.is_empty() {
        return String::new();
    }
    let Some(inst) = s.instances.iter().find(|i| i.name == instance_name) else {
        return String::new();
    };
    let module = library.iter().find(|m| m.name == inst.module_ref);
    let sv = matches!(s.language, Language::SystemVerilog);

    let mut out = String::new();
    if inst.dirty {
        let p = if sv { "//" } else { "--" };
        out += &format!(
            "{p} Source file changed. Review the bindings below;\n\
             {p} ports whose direction/type changed were dropped by re-parse.\n"
        );
    }

    let generics = module.map(|m| m.generics.as_slice()).unwrap_or(&[]);
    let ports: Vec<&str> = module
        .map(|m| m.ports.iter().map(|p| p.name.as_str()).collect())
        .unwrap_or_default();

    let generic_value = |name: &str| -> String {
        inst.generic_map.get(name).cloned().unwrap_or_else(|| {
            generics
                .iter()
                .find(|g| g.name == name)
                .and_then(|g| g.default_value.clone())
                .unwrap_or_default()
        })
    };
    let port_rhs = |name: &str| -> String {
        match inst.port_map.get(name) {
            Some(Some(net)) => rhs_editor_form(net),
            _ => "open".to_string(),
        }
    };
    let port_lhs = |name: &str| -> String {
        match inst.consumer_slices.get(name) {
            Some(sl) => format!("{name}{}", sl.to_suffix()),
            None => name.to_string(),
        }
    };
    let gen_width = generics.iter().map(|g| g.name.len()).max().unwrap_or(0);
    let port_width = ports.iter().map(|p| p.len()).max().unwrap_or(0);

    if sv {
        if !generics.is_empty() {
            out += &format!("{} #(\n", inst.module_ref);
            for g in generics {
                out += &format!("  .{:<gen_width$}({}),\n", g.name, generic_value(&g.name));
            }
            out += &format!(") {} (\n", instance_name);
        } else {
            out += &format!("{} {} (\n", inst.module_ref, instance_name);
        }
        for p in &ports {
            out += &format!("  .{:<port_width$}({}),\n", port_lhs(p), port_rhs(p));
        }
        out += ");\n";
        return out;
    }

    out += &format!("{} : {}\n", instance_name, inst.module_ref);
    if !generics.is_empty() {
        out += "  generic map (\n";
        for g in generics {
            out += &format!("    {:<gen_width$} => {},\n", g.name, generic_value(&g.name));
        }
        out += "  )\n";
    }
    out += "  port map (\n";
    for p in &ports {
        out += &format!("    {:<port_width$} => {},\n", port_lhs(p), port_rhs(p));
    }
    out += "  );\n";
    out
}

fn is_ident(s: &str) -> bool {
    let mut ch = s.chars();
    match ch.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    ch.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate one RHS. Accepted forms: `ident`, `inst.port`, either with a
/// `[i]` / `[h:l]` suffix, or `open` (case-insensitive → empty string).
/// Detect an HDL constant literal on a port-map RHS. Covers VHDL (`'0'`,
/// `"0101"`, `x"AB"`, `123`) and SV (`1'b0`, `8'hFF`, `'0`, `123`) forms.
/// Identifiers can never start with a digit or quote, so this cannot shadow
/// a real port reference.
pub fn is_constant_literal(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if first == '\'' || first == '"' || first.is_ascii_digit() {
        return true;
    }
    // VHDL based bit-string literals: x"AB", B"1010", o"17" ...
    matches!(first.to_ascii_lowercase(), 'x' | 'b' | 'o' | 'd')
        && chars.next() == Some('"')
}

fn validate_rhs(name: &str, rhs: &str) -> Result<String, String> {
    let mut r = rhs.trim();
    while let Some(stripped) = r.strip_suffix(',') {
        r = stripped.trim_end();
    }
    if r.is_empty() {
        return Err(format!("{name}: empty RHS"));
    }
    if r.eq_ignore_ascii_case("open") {
        return Ok(String::new());
    }
    if is_constant_literal(r) {
        return Ok(r.to_string());
    }
    let (head, slice) = match r.find('[') {
        Some(i) => (&r[..i], Some(&r[i..])),
        None => (r, None),
    };
    let head_ok = match head.split_once('.') {
        Some((a, b)) => is_ident(a) && is_ident(b),
        None => is_ident(head),
    };
    let slice_ok = match slice {
        None => true,
        Some(sl) => sl
            .strip_prefix('[')
            .and_then(|x| x.strip_suffix(']'))
            .is_some_and(|inner| match inner.split_once(':') {
                Some((h, l)) => {
                    !h.is_empty()
                        && !l.is_empty()
                        && h.chars().all(|c| c.is_ascii_digit())
                        && l.chars().all(|c| c.is_ascii_digit())
                }
                None => !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit()),
            }),
    };
    if !head_ok || !slice_ok {
        return Err(format!("{name}: cannot parse RHS '{r}'"));
    }
    Ok(r.to_string())
}

/// Extract (lhs, rhs) from a binding line. Recognizes both
/// `name => value[,]` (VHDL) and `.name(value)[,]` (SV). None for
/// comments, blanks, section headers, and closing punctuation.
fn extract_binding(line: &str) -> Option<(String, String)> {
    let s = line.trim();
    if s.is_empty() || s.starts_with("--") || s.starts_with("//") {
        return None;
    }
    if s.starts_with("generic map") || s.starts_with("port map") {
        return None;
    }
    if s == ")" || s == ");" {
        return None;
    }
    // SV header lines like `module_name #(` or `) u_inst (` carry no binding.
    if s.ends_with("#(") || s.ends_with('(') {
        return None;
    }

    if s.starts_with('.') {
        let open = s.find('(')?;
        let close = s.rfind(')')?;
        if open < 2 || close <= open {
            return None;
        }
        let name = s[1..open].trim();
        let val = s[open + 1..close].trim();
        if name.is_empty() {
            return None;
        }
        return Some((name.to_string(), val.to_string()));
    }

    // VHDL form `name => value,` — reject `name : type` lines.
    if s.contains(':') && !s.contains("=>") {
        return None;
    }
    let arrow = s.find("=>")?;
    let lhs = s[..arrow].trim();
    let rhs = s[arrow + 2..].trim();
    if lhs.is_empty() {
        return None;
    }
    Some((lhs.to_string(), rhs.to_string()))
}

// SV section headers, matched without a language hint:
//   `<module> #(`        → starts Generics
//   `) <inst> (`         → ends Generics, starts Ports
//   `<module> <inst> (`  → no-params header, starts Ports
fn is_sv_params_open(s: &str) -> bool {
    let t: Vec<&str> = s.split_whitespace().collect();
    t.len() == 2 && is_ident(t[0]) && t[1] == "#("
}
fn is_sv_params_to_ports(s: &str) -> bool {
    let t: Vec<&str> = s.split_whitespace().collect();
    t.len() == 3 && t[0] == ")" && is_ident(t[1]) && t[2] == "("
}
fn is_sv_ports_only(s: &str) -> bool {
    let t: Vec<&str> = s.split_whitespace().collect();
    t.len() == 3 && is_ident(t[0]) && is_ident(t[1]) && t[2] == "("
}

pub fn parse_instance_buffer(text: &str) -> ParsedBuffer {
    #[derive(PartialEq)]
    enum Section {
        None,
        Generics,
        Ports,
    }
    let mut section = Section::None;
    let mut out = ParsedBuffer::default();

    for (i, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.starts_with("generic map") {
            section = Section::Generics;
            continue;
        }
        if trimmed.starts_with("port map") {
            section = Section::Ports;
            continue;
        }
        if is_sv_params_open(trimmed) {
            section = Section::Generics;
            continue;
        }
        if is_sv_params_to_ports(trimmed) {
            section = Section::Ports;
            continue;
        }
        if section == Section::None && is_sv_ports_only(trimmed) {
            section = Section::Ports;
            continue;
        }

        let Some((lhs, rhs)) = extract_binding(raw) else {
            continue;
        };
        match section {
            Section::Ports => match validate_rhs(&lhs, &rhs) {
                Ok(clean) => out.port_commits.push((lhs, clean)),
                Err(e) => out.errors.push((i, e)),
            },
            Section::Generics => {
                let mut v = rhs.trim();
                while let Some(st) = v.strip_suffix(',') {
                    v = st.trim_end();
                }
                out.generic_commits.push((lhs, v.to_string()));
            }
            Section::None => {}
        }
    }
    out
}

/// Pull an optional `[h:l]` / `[i]` suffix off a port-map LHS. Returns the
/// bare port name plus the slice bounds, or None when malformed.
pub fn split_consumer_slice(lhs: &str) -> Option<(String, Option<(i32, i32)>)> {
    let Some(open) = lhs.find('[') else {
        return Some((lhs.to_string(), None));
    };
    let close = lhs.rfind(']')?;
    if close <= open {
        return None;
    }
    let port = lhs[..open].trim().to_string();
    let inner = lhs[open + 1..close].trim();
    let (high, low) = match inner.split_once(':') {
        Some((h, l)) => (h.trim().parse().ok()?, l.trim().parse().ok()?),
        None => {
            let v: i32 = inner.parse().ok()?;
            (v, v)
        }
    };
    Some((port, Some((high, low))))
}

/// Detect the completion context from the text left of the cursor.
/// Handles both binding shapes: VHDL `name => <cursor>` and SV
/// `.name(<cursor>`.
pub fn completion_context(line_before_cursor: &str) -> CompletionCtx {
    let none = CompletionCtx { kind: 0, prefix: String::new(), instance: String::new() };
    let rhs = if let Some(arrow) = line_before_cursor.find("=>") {
        &line_before_cursor[arrow + 2..]
    } else if line_before_cursor.trim_start().starts_with('.')
        && let Some(open) = line_before_cursor.rfind('(')
        && !line_before_cursor[open..].contains(')')
    {
        &line_before_cursor[open + 1..]
    } else {
        return none;
    };
    // Trailing run of identifier chars (letters, digits, _, .).
    let mut start = 0;
    for (i, c) in rhs.char_indices() {
        if !(c.is_alphanumeric() || c == '_' || c == '.') {
            start = i + c.len_utf8();
        }
    }
    let tail = &rhs[start..];
    match tail.split_once('.') {
        Some((inst, prefix)) => CompletionCtx {
            kind: 2,
            prefix: prefix.to_string(),
            instance: inst.to_string(),
        },
        None => CompletionCtx { kind: 1, prefix: tail.to_string(), instance: String::new() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn fixture() -> (Schematic, Vec<ModuleDef>) {
        let module = ModuleDef {
            name: "fifo".into(),
            generics: vec![GenericDef {
                name: "WIDTH".into(),
                type_name: "integer".into(),
                default_value: Some("8".into()),
            }],
            ports: vec![
                PortDef {
                    name: "clk".into(),
                    direction: Direction::In,
                    port_type: PortType::StdLogic,
                    bundle: None,
                },
                PortDef {
                    name: "din".into(),
                    direction: Direction::In,
                    port_type: PortType::StdLogic,
                    bundle: None,
                },
            ],
            source_path: std::path::PathBuf::new(),
            source_hash: 0,
            dependencies: vec![],
        };
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo").unwrap();
        s.set_port_map_entry("u_fifo", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        (s, vec![module])
    }

    #[test]
    fn render_vhdl_shape() {
        let (s, lib) = fixture();
        let text = render_instance_buffer(&s, &lib, "u_fifo");
        assert!(text.contains("u_fifo : fifo"));
        assert!(text.contains("generic map ("));
        assert!(text.contains("WIDTH => 8,"), "default fills in: {text}");
        assert!(text.contains("clk => clk,"));
        assert!(text.contains("din => open,"));
        assert!(text.ends_with("  );\n"));
    }

    #[test]
    fn constant_rhs_renders_and_parses() {
        let (mut s, lib) = fixture();
        s.set_port_map_entry("u_fifo", "din", Some(NetRef::Constant("'0'".into())))
            .unwrap();
        let text = render_instance_buffer(&s, &lib, "u_fifo");
        assert!(text.contains("din => '0',"), "{text}");
        let parsed = parse_instance_buffer(&text);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(
            parsed
                .port_commits
                .iter()
                .any(|(l, r)| l == "din" && r == "'0'")
        );
    }

    #[test]
    fn constant_literal_detection() {
        for lit in ["'0'", "\"0101\"", "x\"AB\"", "X\"ff\"", "8'hFF", "1'b0", "42", "'0"] {
            assert!(is_constant_literal(lit), "{lit}");
        }
        for ident in ["clk", "u_a.dout", "bus[3]", "open_bar", "x_state", "b2b"] {
            assert!(!is_constant_literal(ident), "{ident}");
        }
    }

    #[test]
    fn render_unknown_instance_is_empty() {
        let (s, lib) = fixture();
        assert_eq!(render_instance_buffer(&s, &lib, "nope"), "");
        assert_eq!(render_instance_buffer(&s, &lib, ""), "");
    }

    #[test]
    fn render_slices_in_parseable_form() {
        let (mut s, lib) = fixture();
        s.set_port_map_entry(
            "u_fifo",
            "din",
            Some(NetRef::TopPortSlice("bus".into(), SliceExpr::Bit(0))),
        )
        .unwrap();
        let text = render_instance_buffer(&s, &lib, "u_fifo");
        // The old renderer emitted "top:bus[0]" here, which the parser
        // rejected — the buffer must re-parse cleanly.
        assert!(text.contains("din => bus[0],"), "{text}");
        let parsed = parse_instance_buffer(&text);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    }

    #[test]
    fn render_parse_round_trip() {
        let (s, lib) = fixture();
        let text = render_instance_buffer(&s, &lib, "u_fifo");
        let parsed = parse_instance_buffer(&text);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.generic_commits, vec![("WIDTH".into(), "8".into())]);
        assert_eq!(
            parsed.port_commits,
            vec![("clk".into(), "clk".into()), ("din".into(), String::new())]
        );
    }

    #[test]
    fn parse_sv_form() {
        let text = "fifo #(\n  .WIDTH(16),\n) u_fifo (\n  .clk(clk_sys),\n  .din(u_a.dout[3]),\n);\n";
        let parsed = parse_instance_buffer(text);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert_eq!(parsed.generic_commits, vec![("WIDTH".into(), "16".into())]);
        assert_eq!(
            parsed.port_commits,
            vec![
                ("clk".into(), "clk_sys".into()),
                ("din".into(), "u_a.dout[3]".into())
            ]
        );
    }

    #[test]
    fn parse_flags_bad_rhs_with_line() {
        let text = "u_fifo : fifo\n  port map (\n    clk => cl k!,\n    din => open,\n  );\n";
        let parsed = parse_instance_buffer(text);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].0, 2, "0-based line of the bad binding");
        assert_eq!(parsed.port_commits.len(), 1, "good lines still parse");
    }

    #[test]
    fn open_is_case_insensitive() {
        for text in ["open", "OPEN", "Open"] {
            let buf = format!("u : m\n  port map (\n    a => {text},\n  );\n");
            let parsed = parse_instance_buffer(&buf);
            assert!(parsed.errors.is_empty());
            assert_eq!(parsed.port_commits[0].1, "", "{text} must mean open");
        }
    }

    #[test]
    fn consumer_slice_split() {
        assert_eq!(split_consumer_slice("din"), Some(("din".into(), None)));
        assert_eq!(split_consumer_slice("din[3]"), Some(("din".into(), Some((3, 3)))));
        assert_eq!(split_consumer_slice("din[7:4]"), Some(("din".into(), Some((7, 4)))));
        assert_eq!(split_consumer_slice("din[x]"), None);
        assert_eq!(split_consumer_slice("din[3"), None);
    }

    #[test]
    fn completion_contexts() {
        assert_eq!(completion_context("    clk => ").kind, 1);
        let c = completion_context("    din => u_a.do");
        assert_eq!((c.kind, c.instance.as_str(), c.prefix.as_str()), (2, "u_a", "do"));
        assert_eq!(completion_context("no arrow here").kind, 0);
        let c = completion_context("  x => u_cnt");
        assert_eq!((c.kind, c.prefix.as_str()), (1, "u_cnt"));
    }

    #[test]
    fn completion_contexts_sv_form() {
        assert_eq!(completion_context("  .clk(").kind, 1);
        let c = completion_context("  .din(u_a.do");
        assert_eq!((c.kind, c.instance.as_str(), c.prefix.as_str()), (2, "u_a", "do"));
        let c = completion_context("  .clk(clk_s");
        assert_eq!((c.kind, c.prefix.as_str()), (1, "clk_s"));
        // A closed binding on the same line offers nothing.
        assert_eq!(completion_context("  .clk(clk_sys),").kind, 0);
        // Bare paren without the SV `.name(` shape offers nothing.
        assert_eq!(completion_context("  foo (").kind, 0);
    }

    #[test]
    fn generic_map_lines_not_treated_as_ports() {
        // `name : type` lines and section punctuation must not bind.
        let text = "u : m\n  generic map (\n    DEPTH => 64,\n  )\n  port map (\n  );\n";
        let parsed = parse_instance_buffer(text);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.generic_commits, vec![("DEPTH".into(), "64".into())]);
        assert!(parsed.port_commits.is_empty());
    }
}
