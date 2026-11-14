use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ParseError;
use crate::types::*;

// --- Diagnostic types ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    /// Optional context: which instance/port is involved
    pub instance: Option<String>,
    pub port: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Error,
            message: message.into(),
            instance: None,
            port: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            instance: None,
            port: None,
        }
    }

    fn with_instance(mut self, name: &str) -> Self {
        self.instance = Some(name.to_string());
        self
    }

    fn with_port(mut self, name: &str) -> Self {
        self.port = Some(name.to_string());
        self
    }

    pub fn is_error(&self) -> bool {
        self.level == DiagnosticLevel::Error
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
        };
        match (&self.instance, &self.port) {
            (Some(inst), Some(port)) => write!(f, "{prefix}: {inst}.{port}: {}", self.message),
            (Some(inst), None) => write!(f, "{prefix}: {inst}: {}", self.message),
            _ => write!(f, "{prefix}: {}", self.message),
        }
    }
}

// --- Schematic error ---

#[derive(Debug, thiserror::Error)]
pub enum SchematicError {
    #[error("duplicate instance name: {0}")]
    DuplicateInstance(String),

    #[error("instance not found: {0}")]
    InstanceNotFound(String),
}

// --- Schematic operations ---

impl Schematic {
    pub fn new(top_name: impl Into<String>, language: Language) -> Self {
        Schematic {
            top_name: top_name.into(),
            language,
            top_generics: Vec::new(),
            top_ports: Vec::new(),
            instances: Vec::new(),
            aliases: HashMap::new(),
            library_paths: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn add_instance(
        &mut self,
        name: impl Into<String>,
        module_ref: impl Into<String>,
    ) -> Result<&mut Instance, SchematicError> {
        let name = name.into();
        if self.instances.iter().any(|i| i.name == name) {
            return Err(SchematicError::DuplicateInstance(name));
        }
        self.instances.push(Instance {
            name,
            module_ref: module_ref.into(),
            generic_map: HashMap::new(),
            port_map: HashMap::new(),
            position: (0.0, 0.0),
            manual_bundles: HashMap::new(),
            consumer_slices: HashMap::new(),
            dirty: false,
            locked: false,
        });
        Ok(self.instances.last_mut().unwrap())
    }

    pub fn rename_instance(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), SchematicError> {
        if self.instances.iter().any(|i| i.name == new_name) {
            return Err(SchematicError::DuplicateInstance(new_name.to_string()));
        }
        let inst = self
            .instances
            .iter_mut()
            .find(|i| i.name == old_name)
            .ok_or_else(|| SchematicError::InstanceNotFound(old_name.to_string()))?;
        inst.name = new_name.to_string();
        // Rewrite any port_map entries that referenced the old name.
        for other in self.instances.iter_mut() {
            for entry in other.port_map.values_mut() {
                match entry {
                    Some(NetRef::InstancePort(inst, _)) if inst == old_name => {
                        *inst = new_name.to_string();
                    }
                    Some(NetRef::InstancePortSlice(inst, _, _)) if inst == old_name => {
                        *inst = new_name.to_string();
                    }
                    _ => {}
                }
            }
        }
        // Group membership follows the rename.
        for g in self.groups.iter_mut() {
            for m in g.members.iter_mut() {
                if m == old_name {
                    *m = new_name.to_string();
                }
            }
        }
        // Rewrite alias keys if any reference this instance as driver.
        let remapped: Vec<(NetRef, String)> = self
            .aliases
            .iter()
            .filter_map(|(k, v)| match k {
                NetRef::InstancePort(inst, port) if inst == old_name => Some((
                    NetRef::InstancePort(new_name.to_string(), port.clone()),
                    v.clone(),
                )),
                _ => None,
            })
            .collect();
        self.aliases
            .retain(|k, _| !matches!(k, NetRef::InstancePort(inst, _) if inst == old_name));
        for (k, v) in remapped {
            self.aliases.insert(k, v);
        }
        Ok(())
    }

    /// Diff against an older version of the library and update instance
    /// port_maps. For every instance whose module's port list has changed,
    /// drop port_map entries for ports that no longer exist on the module
    /// and set the instance's `dirty` flag. Returns the names of newly-dirty
    /// instances so the caller can surface a notification.
    pub fn apply_library_update(
        &mut self,
        old_library: &[ModuleDef],
        new_library: &[ModuleDef],
    ) -> Vec<String> {
        let old_map: HashMap<&str, &ModuleDef> =
            old_library.iter().map(|m| (m.name.as_str(), m)).collect();
        let new_map: HashMap<&str, &ModuleDef> =
            new_library.iter().map(|m| (m.name.as_str(), m)).collect();

        let mut newly_dirty = Vec::new();
        for inst in self.instances.iter_mut() {
            let Some(new_module) = new_map.get(inst.module_ref.as_str()) else {
                continue;
            };
            let new_port_names: HashSet<&str> =
                new_module.ports.iter().map(|p| p.name.as_str()).collect();
            // Ports that existed before but are gone now (or whose direction /
            // type changed enough that the old connection is suspect).
            let mut dropped_ports = Vec::new();
            let old_module = old_map.get(inst.module_ref.as_str());
            if let Some(old) = old_module {
                for old_port in &old.ports {
                    if !new_port_names.contains(old_port.name.as_str()) {
                        dropped_ports.push(old_port.name.clone());
                        continue;
                    }
                    // Port still exists — check direction / type for breaking change.
                    if let Some(new_port) =
                        new_module.ports.iter().find(|p| p.name == old_port.name)
                        && (new_port.direction != old_port.direction
                            || new_port.port_type != old_port.port_type)
                    {
                        dropped_ports.push(old_port.name.clone());
                    }
                }
            }
            if dropped_ports.is_empty() {
                continue;
            }
            for p in &dropped_ports {
                inst.port_map.remove(p);
                inst.consumer_slices.remove(p);
            }
            inst.dirty = true;
            newly_dirty.push(inst.name.clone());
        }
        newly_dirty
    }

    /// Clear the dirty flag on an instance.
    pub fn clear_instance_dirty(&mut self, name: &str) -> bool {
        if let Some(inst) = self.get_instance_mut(name) {
            let was_dirty = inst.dirty;
            inst.dirty = false;
            return was_dirty;
        }
        false
    }

    /// Sweep all port_map entries and aliases for references to instances that
    /// do not exist. Used on project load to clean up stale data from old files
    /// saved before `remove_instance` swept sibling references. Returns the
    /// number of entries cleared so the caller can surface a warning.
    pub fn cleanup_stale_refs(&mut self) -> usize {
        let live: HashSet<String> = self.instances.iter().map(|i| i.name.clone()).collect();
        let mut cleared = 0;
        for inst in self.instances.iter_mut() {
            for entry in inst.port_map.values_mut() {
                if let Some(net) = entry
                    && let Some(ref_inst) = net.instance_name()
                    && !live.contains(ref_inst)
                {
                    *entry = None;
                    cleared += 1;
                }
            }
        }
        let before = self.aliases.len();
        self.aliases.retain(|k, _| match k.instance_name() {
            Some(inst) => live.contains(inst),
            None => true,
        });
        cleared += before - self.aliases.len();
        cleared
    }

    /// Atomically replace the top-level entity declaration. Cascades through
    /// `instances.port_map` clearing any entries that reference top ports
    /// removed by this swap. Returns the names of removed top ports.
    pub fn replace_top_level(
        &mut self,
        new_top_name: String,
        new_generics: Vec<GenericDef>,
        new_ports: Vec<PortDef>,
    ) -> Vec<String> {
        let old_names: HashSet<String> = self.top_ports.iter().map(|p| p.name.clone()).collect();
        let new_names: HashSet<String> = new_ports.iter().map(|p| p.name.clone()).collect();
        let removed: Vec<String> = old_names.difference(&new_names).cloned().collect();
        for inst in self.instances.iter_mut() {
            for entry in inst.port_map.values_mut() {
                if let Some(net) = entry {
                    let base = net.base();
                    if let NetRef::TopPort(name) = &base
                        && removed.iter().any(|r| r == name)
                    {
                        *entry = None;
                    }
                }
            }
        }
        self.aliases.retain(|k, _| match k {
            NetRef::TopPort(name) | NetRef::TopPortSlice(name, _) => !removed.contains(name),
            _ => true,
        });
        self.top_name = new_top_name;
        self.top_generics = new_generics;
        self.top_ports = new_ports;
        removed
    }

    /// Remove an instance, and clear any port_map entries or aliases that referenced it.
    pub fn remove_instance(&mut self, name: &str) -> Result<Instance, SchematicError> {
        let idx = self
            .instances
            .iter()
            .position(|i| i.name == name)
            .ok_or_else(|| SchematicError::InstanceNotFound(name.to_string()))?;
        let removed = self.instances.remove(idx);
        // Clear any sibling port_map entries that referenced the removed instance.
        for other in self.instances.iter_mut() {
            for entry in other.port_map.values_mut() {
                if entry.as_ref().and_then(|nr| nr.instance_name()) == Some(name) {
                    *entry = None;
                }
            }
        }
        // Drop any aliases whose key referenced the removed instance as driver.
        self.aliases.retain(|k, _| k.instance_name() != Some(name));
        // Group membership follows the instance; empty groups dissolve.
        for g in self.groups.iter_mut() {
            g.members.retain(|m| m != name);
        }
        let emptied: Vec<String> = self
            .groups
            .iter()
            .filter(|g| g.members.is_empty())
            .map(|g| g.name.clone())
            .collect();
        for gone in emptied {
            for g in self.groups.iter_mut() {
                if g.parent.as_deref() == Some(gone.as_str()) {
                    g.parent = None;
                }
            }
            self.groups.retain(|g| g.name != gone);
        }
        Ok(removed)
    }

    pub fn get_instance(&self, name: &str) -> Option<&Instance> {
        self.instances.iter().find(|i| i.name == name)
    }

    pub fn get_instance_mut(&mut self, name: &str) -> Option<&mut Instance> {
        self.instances.iter_mut().find(|i| i.name == name)
    }

    pub fn set_port_map_entry(
        &mut self,
        instance_name: &str,
        port_name: impl Into<String>,
        net_ref: Option<NetRef>,
    ) -> Result<(), SchematicError> {
        let inst = self
            .get_instance_mut(instance_name)
            .ok_or_else(|| SchematicError::InstanceNotFound(instance_name.to_string()))?;
        inst.port_map.insert(port_name.into(), net_ref);
        Ok(())
    }

    pub fn set_generic_map_entry(
        &mut self,
        instance_name: &str,
        generic_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), SchematicError> {
        let inst = self
            .get_instance_mut(instance_name)
            .ok_or_else(|| SchematicError::InstanceNotFound(instance_name.to_string()))?;
        inst.generic_map.insert(generic_name.into(), value.into());
        Ok(())
    }

    pub fn add_library_path(&mut self, path: std::path::PathBuf) -> bool {
        if self.library_paths.iter().any(|p| p == &path) {
            return false;
        }
        self.library_paths.push(path);
        true
    }

    pub fn remove_library_path(&mut self, path: &std::path::Path) -> bool {
        let before = self.library_paths.len();
        self.library_paths.retain(|p| p != path);
        before != self.library_paths.len()
    }

    pub fn set_alias(&mut self, net_id: NetId, alias: impl Into<String>) {
        self.aliases.insert(net_id, alias.into());
    }

    pub fn remove_alias(&mut self, net_id: &NetId) {
        self.aliases.remove(net_id);
    }

    /// Resolve module references: parse every library path. Paths that fail
    /// to parse are returned as errors alongside the modules that did parse —
    /// one bad source must not wipe the entire library.
    pub fn resolve_modules(&self) -> (Vec<ModuleDef>, Vec<(PathBuf, ParseError)>) {
        let mut modules = Vec::new();
        let mut errors = Vec::new();
        for path in &self.library_paths {
            match crate::parse_file(path) {
                Ok(defs) => modules.extend(defs),
                Err(e) => errors.push((path.clone(), e)),
            }
        }
        (modules, errors)
    }

    /// Get the BASE signal name for a net — alias if set, otherwise derived from
    /// the driver pin. Slice suffixes are NOT appended; callers that need a
    /// language-specific slice render (VHDL `(h downto l)` / SV `[h:l]`) look
    /// at the `NetRef` variant themselves and format it.
    pub fn signal_name(&self, net_ref: &NetRef) -> String {
        let base = net_ref.base();
        if let Some(alias) = self.aliases.get(&base) {
            return alias.clone();
        }
        match &base {
            NetRef::TopPort(name) => {
                // InOut top ports cannot route through an intermediate signal
                // (`<port> <= <sig>; <sig> <= <port>;` would be a multi-driver
                // conflict), so keep direct naming for them.
                if let Some(p) = self.top_ports.iter().find(|p| &p.name == name)
                    && p.direction == Direction::InOut
                {
                    return name.clone();
                }
                format!("{name}_s")
            }
            NetRef::InstancePort(inst, port) => format!("{inst}_{port}"),
            _ => unreachable!("base() always returns non-slice variant"),
        }
    }

    /// Validate the schematic against a resolved module library.
    pub fn validate(&self, library: &[ModuleDef]) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lib_map: HashMap<&str, &ModuleDef> =
            library.iter().map(|m| (m.name.as_str(), m)).collect();
        // Resolved net view: connectivity is net membership, not entry
        // presence — a port referenced by another pin is connected even
        // with no port_map entry of its own.
        let nets = crate::nets::resolve_nets(self, library);

        // Duplicate instance names
        {
            let mut seen = HashSet::new();
            for inst in &self.instances {
                if !seen.insert(&inst.name) {
                    diags.push(
                        Diagnostic::error(format!("duplicate instance name '{}'", inst.name))
                            .with_instance(&inst.name),
                    );
                }
            }
        }

        // Per-instance validation
        let instance_names: HashSet<&str> =
            self.instances.iter().map(|i| i.name.as_str()).collect();

        for inst in &self.instances {
            // Unresolved module reference
            let module_def = match lib_map.get(inst.module_ref.as_str()) {
                Some(m) => Some(*m),
                None => {
                    diags.push(
                        Diagnostic::error(format!(
                            "module '{}' not found in library",
                            inst.module_ref
                        ))
                        .with_instance(&inst.name),
                    );
                    None
                }
            };

            let module_ports: HashMap<&str, &PortDef> = module_def
                .map(|m| m.ports.iter().map(|p| (p.name.as_str(), p)).collect())
                .unwrap_or_default();

            // Check each port map entry
            for (port_name, net_ref_opt) in &inst.port_map {
                // A key that is not a port of the module. Codegen has no
                // port to put the entry on, so the connection would vanish.
                if let Some(m) = module_def
                    && !module_ports.contains_key(port_name.as_str())
                {
                    diags.push(
                        Diagnostic::error(format!(
                            "port '{port_name}' does not exist on module '{}'",
                            m.name
                        ))
                        .with_instance(&inst.name)
                        .with_port(port_name),
                    );
                    continue;
                }

                // The type of this pin: the consumer slice if there is
                // one, else the port with its generics evaluated.
                let pin_type = module_def.and_then(|m| {
                    let port = module_ports.get(port_name.as_str())?;
                    Some(crate::nets::instance_pin_type(inst, m, port))
                });

                let Some(net_ref) = net_ref_opt else {
                    // Explicitly unconnected — but only warn when no other
                    // pin pulled this port onto a net.
                    let pin = NetRef::InstancePort(inst.name.clone(), port_name.clone());
                    if nets.net_for(&pin).is_none() {
                        diags.push(
                            unconnected_diag(
                                port_name,
                                module_ports.get(port_name.as_str()).copied(),
                            )
                            .with_instance(&inst.name)
                            .with_port(port_name),
                        );
                    }
                    continue;
                };

                // Constant ties: legal on inputs, meaningless on outputs.
                if let NetRef::Constant(lit) = net_ref {
                    if module_ports
                        .get(port_name.as_str())
                        .is_some_and(|p| p.direction == Direction::Out)
                    {
                        diags.push(
                            Diagnostic::error(format!(
                                "output port '{port_name}' cannot be tied to constant {lit}"
                            ))
                            .with_instance(&inst.name)
                            .with_port(port_name),
                        );
                    }
                    continue;
                }

                // Normalize the driver reference to base (inst/port or top) plus an optional slice.
                let (top_name_opt, inst_ref_opt, slice_opt) = match net_ref {
                    NetRef::TopPort(n) => (Some(n.clone()), None, None),
                    NetRef::TopPortSlice(n, s) => (Some(n.clone()), None, Some(s)),
                    NetRef::InstancePort(i, p) => (None, Some((i.clone(), p.clone())), None),
                    NetRef::InstancePortSlice(i, p, s) => {
                        (None, Some((i.clone(), p.clone())), Some(s))
                    }
                    NetRef::Constant(_) => unreachable!("handled above"),
                };

                // The type of the referenced pin, if we can find one. Used for slice bounds.
                let mut driver_type: Option<PortType> = None;

                if let Some(top_name) = &top_name_opt {
                    // Check top port exists
                    if !self.top_ports.iter().any(|p| p.name == *top_name) {
                        diags.push(
                            Diagnostic::error(format!(
                                "references top-level port '{top_name}' which does not exist"
                            ))
                            .with_instance(&inst.name)
                            .with_port(port_name),
                        );
                    } else if let Some(pin_type) = &pin_type
                        && let Some(top_port) = self.top_ports.iter().find(|p| p.name == *top_name)
                    {
                        driver_type = Some(top_port.port_type.clone());
                        if slice_opt.is_none() {
                            check_compatibility(
                                &mut diags,
                                &inst.name,
                                port_name,
                                pin_type,
                                &top_port.name,
                                &top_port.port_type,
                                &self.top_generics,
                            );
                        }
                    }
                } else if let Some((ref_inst, ref_port)) = &inst_ref_opt {
                    if !instance_names.contains(ref_inst.as_str()) {
                        diags.push(
                            Diagnostic::error(format!(
                                "references instance '{ref_inst}' which does not exist"
                            ))
                            .with_instance(&inst.name)
                            .with_port(port_name),
                        );
                        continue;
                    }

                    let ref_instance = self.instances.iter().find(|i| i.name == *ref_inst).unwrap();
                    if let Some(ref_module) = lib_map.get(ref_instance.module_ref.as_str()) {
                        if let Some(driver_port) =
                            ref_module.ports.iter().find(|p| p.name == *ref_port)
                        {
                            // The bits that the referenced instance
                            // connects: its generics and its consumer
                            // slice count, not the port as written.
                            let referenced = crate::nets::instance_pin_type(
                                ref_instance,
                                ref_module,
                                driver_port,
                            );
                            // Driver presence is checked net-wide below —
                            // referencing another input is fine as long as
                            // the merged net has a driver somewhere.
                            if let Some(pin_type) = &pin_type
                                && slice_opt.is_none()
                            {
                                check_compatibility(
                                    &mut diags,
                                    &inst.name,
                                    port_name,
                                    pin_type,
                                    &driver_port.name,
                                    &referenced,
                                    &self.top_generics,
                                );
                            }
                            driver_type = Some(referenced);
                        } else {
                            diags.push(
                                Diagnostic::error(format!(
                                    "port '{ref_port}' does not exist on module '{}'",
                                    ref_instance.module_ref
                                ))
                                .with_instance(&inst.name)
                                .with_port(port_name),
                            );
                        }
                    }
                }

                // Slice-range check against the width of the referenced pin.
                if let (Some(slice), Some(driver_type)) = (slice_opt, &driver_type)
                    && let Some(dw) = width_at_defaults(driver_type, &self.top_generics)
                {
                    let (hi, lo) = match slice {
                        SliceExpr::Bit(i) => (*i, *i),
                        SliceExpr::Range { high, low } => (*high, *low),
                    };
                    if lo < 0 || hi < lo || (hi as u32) >= dw {
                        diags.push(
                            Diagnostic::error(format!(
                                "slice [{hi}:{lo}] is out of range for driver of width {dw}"
                            ))
                            .with_instance(&inst.name)
                            .with_port(port_name),
                        );
                    }
                }

                // A reference with a slice connects the pin to the bits of
                // the slice.
                if let Some(slice) = slice_opt
                    && let Some(pin_width) = pin_type
                        .as_ref()
                        .and_then(|t| width_at_defaults(t, &self.top_generics))
                    && pin_width != slice_width(slice)
                {
                    diags.push(
                        Diagnostic::error(format!(
                            "width mismatch: port is {pin_width} bits but the slice {} is {} bits",
                            slice.to_suffix(),
                            slice_width(slice)
                        ))
                        .with_instance(&inst.name)
                        .with_port(port_name),
                    );
                }
            }

            // Check for ports in module that aren't in port_map at all (implicit unconnected)
            if let Some(m) = module_def {
                for port in &m.ports {
                    let pin = NetRef::InstancePort(inst.name.clone(), port.name.clone());
                    if !inst.port_map.contains_key(&port.name) && nets.net_for(&pin).is_none() {
                        diags.push(
                            unconnected_diag(&port.name, Some(port))
                                .with_instance(&inst.name)
                                .with_port(&port.name),
                        );
                    }
                }
                // A generic_map key that is not a generic of the module.
                // Codegen would emit it, and the HDL tool would reject it.
                for generic_name in inst.generic_map.keys() {
                    if !m.generics.iter().any(|g| &g.name == generic_name) {
                        diags.push(
                            Diagnostic::error(format!(
                                "generic '{generic_name}' does not exist on module '{}'",
                                m.name
                            ))
                            .with_instance(&inst.name),
                        );
                    }
                }
                // Manual bundles must reference ports that exist on the module.
                let module_port_names: HashSet<&str> =
                    m.ports.iter().map(|p| p.name.as_str()).collect();
                for (bundle_name, ports) in &inst.manual_bundles {
                    for port_name in ports {
                        if !module_port_names.contains(port_name.as_str()) {
                            diags.push(
                                Diagnostic::error(format!(
                                    "manual bundle '{bundle_name}' references port '{port_name}' which does not exist on module '{}'",
                                    m.name
                                ))
                                .with_instance(&inst.name),
                            );
                        }
                    }
                }
            }
        }

        // A top-level port that no port_map entry references. The generated
        // module would have the port with nothing behind it.
        for port in &self.top_ports {
            if nets.net_for(&NetRef::TopPort(port.name.clone())).is_none() {
                diags.push(Diagnostic::warning(format!(
                    "top-level port '{}' is unconnected",
                    port.name
                )));
            }
        }

        // Net-level driver checks over the resolved (merged) nets.
        {
            for net in &nets.nets {
                // InOut pins may legitimately share a net (tri-state bus);
                // more than one hard driver is a conflict.
                let hard_drivers: Vec<&NetRef> = net
                    .drivers
                    .iter()
                    .filter(|r| {
                        crate::nets::pin_direction(self, &lib_map, r) != Some(Direction::InOut)
                    })
                    .collect();
                // Disjoint slices of one vector port are legitimate fan-in
                // (led[0]/led[1]/... from different instances); only
                // overlapping bits conflict.
                let mut conflicting: Vec<&NetRef> = Vec::new();
                for (i, a) in hard_drivers.iter().enumerate() {
                    for b in &hard_drivers[i + 1..] {
                        if crate::nets::drivers_conflict(self, a, b) {
                            if !conflicting.contains(a) {
                                conflicting.push(a);
                            }
                            if !conflicting.contains(b) {
                                conflicting.push(b);
                            }
                        }
                    }
                }
                if !conflicting.is_empty() {
                    let pins: Vec<String> = conflicting.iter().map(|r| r.to_key()).collect();
                    diags.push(Diagnostic::error(format!(
                        "net '{}' has multiple drivers: {}",
                        net.name,
                        pins.join(", ")
                    )));
                }
                if net.drivers.is_empty() {
                    let pins: Vec<String> = net.members.iter().map(|r| r.to_key()).collect();
                    diags.push(Diagnostic::warning(format!(
                        "net '{}' has no driver (pins: {})",
                        net.name,
                        pins.join(", ")
                    )));
                }
                // One net, one name: conflicting aliases on merged pins.
                let aliases = net.aliases(self);
                if aliases.len() > 1 {
                    diags.push(Diagnostic::error(format!(
                        "net '{}' has conflicting aliases: {}",
                        net.name,
                        aliases.join(", ")
                    )));
                }
            }
        }

        // Duplicate alias names
        {
            let mut alias_names: HashMap<&str, Vec<&NetId>> = HashMap::new();
            for (net_id, alias) in &self.aliases {
                alias_names.entry(alias.as_str()).or_default().push(net_id);
            }
            for (alias, net_ids) in &alias_names {
                if net_ids.len() > 1 {
                    diags.push(Diagnostic::error(format!(
                        "alias '{alias}' is used for {} different nets",
                        net_ids.len()
                    )));
                }
            }
        }

        diags
    }
}

/// Warning for an unconnected port; inputs get the sharper message because
/// VHDL rejects `open` on inputs without default values.
fn unconnected_diag(port_name: &str, port_def: Option<&PortDef>) -> Diagnostic {
    match port_def.map(|p| &p.direction) {
        Some(Direction::In) => Diagnostic::warning(format!(
            "input port '{port_name}' is unconnected — VHDL rejects open inputs \
             without defaults; connect it or tie it to a constant"
        )),
        _ => Diagnostic::warning(format!("port '{port_name}' is unconnected")),
    }
}

/// The width of a type. A bound that uses a top-level generic is evaluated
/// at the default value of that generic: `check`, `synth` and `build`
/// elaborate the generated top level with the default values, so a
/// mismatch there is a defect of the project. Returns None when a bound has
/// no known value.
fn width_at_defaults(port_type: &PortType, top_generics: &[GenericDef]) -> Option<u32> {
    port_width(&crate::codegen::resolve_port_type(
        port_type,
        top_generics,
        &HashMap::new(),
    ))
}

/// Check width compatibility between the types of two connected ports.
fn check_compatibility(
    diags: &mut Vec<Diagnostic>,
    inst_name: &str,
    port_name: &str,
    consumer: &PortType,
    driver_name: &str,
    driver: &PortType,
    top_generics: &[GenericDef],
) {
    // Both widths resolved and unequal -> mismatch
    if let (Some(cw), Some(dw)) = (
        width_at_defaults(consumer, top_generics),
        width_at_defaults(driver, top_generics),
    ) && cw != dw
    {
        // Tell the user when the numbers come from a default value.
        let note = if port_width(consumer).is_none() || port_width(driver).is_none() {
            " at the default value of the top-level generics"
        } else {
            ""
        };
        diags.push(
            Diagnostic::error(format!(
                "width mismatch: port is {cw} bits but driver '{driver_name}' is {dw} bits{note}"
            ))
            .with_instance(inst_name)
            .with_port(port_name),
        );
        return;
    }

    // Scalar-vs-vector mismatch, even when the vector width is unresolved.
    let consumer_is_scalar = matches!(consumer, PortType::StdLogic);
    let driver_is_scalar = matches!(driver, PortType::StdLogic);
    let consumer_is_vector = matches!(consumer, PortType::StdLogicVector(_));
    let driver_is_vector = matches!(driver, PortType::StdLogicVector(_));
    if (consumer_is_scalar && driver_is_vector) || (consumer_is_vector && driver_is_scalar) {
        diags.push(
            Diagnostic::error(format!(
                "type mismatch: scalar cannot drive or be driven by vector '{driver_name}'"
            ))
            .with_instance(inst_name)
            .with_port(port_name),
        );
    }
}

/// Number of bits that a slice selects.
fn slice_width(slice: &SliceExpr) -> u32 {
    match slice {
        SliceExpr::Bit(_) => 1,
        SliceExpr::Range { high, low } => high.abs_diff(*low) + 1,
    }
}

/// Try to compute a concrete width from a PortType. Returns None if parameterized.
fn port_width(pt: &PortType) -> Option<u32> {
    match pt {
        PortType::StdLogic => Some(1),
        PortType::StdLogicVector(range) => {
            if let (RangeExpr::Literal(h), RangeExpr::Literal(l)) = (&range.high, &range.low) {
                Some((h - l).unsigned_abs() as u32 + 1)
            } else {
                None // parameterized, can't check
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_module(name: &str, ports: Vec<PortDef>) -> ModuleDef {
        ModuleDef {
            name: name.to_string(),
            generics: Vec::new(),
            ports,
            source_path: "test.vhd".into(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    fn make_port(name: &str, dir: Direction, pt: PortType) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction: dir,
            port_type: pt,
            bundle: None,
        }
    }

    #[test]
    fn new_schematic() {
        let s = Schematic::new("top", Language::Vhdl);
        assert_eq!(s.top_name, "top");
        assert_eq!(s.language, Language::Vhdl);
        assert!(s.instances.is_empty());
    }

    #[test]
    fn add_and_remove_instance() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        assert_eq!(s.instances.len(), 1);
        assert_eq!(s.instances[0].name, "u_fifo");

        let removed = s.remove_instance("u_fifo").unwrap();
        assert_eq!(removed.name, "u_fifo");
        assert!(s.instances.is_empty());
    }

    #[test]
    fn duplicate_instance_rejected() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        let err = s.add_instance("u_fifo", "fifo_sync").unwrap_err();
        assert!(matches!(err, SchematicError::DuplicateInstance(_)));
    }

    #[test]
    fn set_port_and_generic_map() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo_sync").unwrap();

        s.set_port_map_entry("u_fifo", "clk", Some(NetRef::TopPort("clk_sys".into())))
            .unwrap();
        s.set_generic_map_entry("u_fifo", "DEPTH", "1024").unwrap();

        let inst = s.get_instance("u_fifo").unwrap();
        assert_eq!(
            inst.port_map.get("clk"),
            Some(&Some(NetRef::TopPort("clk_sys".into())))
        );
        assert_eq!(inst.generic_map.get("DEPTH"), Some(&"1024".to_string()));
    }

    #[test]
    fn aliases() {
        let mut s = Schematic::new("top", Language::Vhdl);
        let net = NetRef::InstancePort("u_pll".into(), "clk_out".into());

        assert_eq!(s.signal_name(&net), "u_pll_clk_out");

        s.set_alias(net.clone(), "sys_clk");
        assert_eq!(s.signal_name(&net), "sys_clk");

        s.remove_alias(&net);
        assert_eq!(s.signal_name(&net), "u_pll_clk_out");
    }

    #[test]
    fn validate_missing_instance_ref() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        s.set_port_map_entry(
            "u_fifo",
            "din",
            Some(NetRef::InstancePort("u_ghost".into(), "data".into())),
        )
        .unwrap();

        let lib = vec![make_module(
            "fifo_sync",
            vec![make_port("din", Direction::In, PortType::StdLogic)],
        )];
        let diags = s.validate(&lib);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("u_ghost"))
        );
    }

    #[test]
    fn validate_missing_port_ref() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_adc", "adc_module").unwrap();
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        s.set_port_map_entry(
            "u_fifo",
            "din",
            Some(NetRef::InstancePort("u_adc".into(), "nonexistent".into())),
        )
        .unwrap();

        let lib = vec![
            make_module(
                "adc_module",
                vec![make_port("data_out", Direction::Out, PortType::StdLogic)],
            ),
            make_module(
                "fifo_sync",
                vec![make_port("din", Direction::In, PortType::StdLogic)],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("nonexistent"))
        );
    }

    #[test]
    fn validate_input_referenced_net_is_warning_not_error() {
        // u_b.din references u_a.data_in — both inputs, i.e. shared undriven
        // signal net. Should warn, not error, so codegen still runs.
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "data_in".into())),
        )
        .unwrap();

        let lib = vec![
            make_module(
                "mod_a",
                vec![make_port("data_in", Direction::In, PortType::StdLogic)],
            ),
            make_module(
                "mod_b",
                vec![make_port("din", Direction::In, PortType::StdLogic)],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(diags.iter().all(|d| !d.is_error()));
        assert!(
            diags
                .iter()
                .any(|d| !d.is_error() && d.message.contains("no driver"))
        );
    }

    #[test]
    fn validate_width_mismatch() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();

        let lib = vec![
            make_module(
                "mod_a",
                vec![make_port(
                    "dout",
                    Direction::Out,
                    PortType::StdLogicVector(Range {
                        high: RangeExpr::Literal(15),
                        low: RangeExpr::Literal(0),
                        dir: RangeDir::Downto,
                    }),
                )],
            ),
            make_module(
                "mod_b",
                vec![make_port(
                    "din",
                    Direction::In,
                    PortType::StdLogicVector(Range {
                        high: RangeExpr::Literal(7),
                        low: RangeExpr::Literal(0),
                        dir: RangeDir::Downto,
                    }),
                )],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("width mismatch"))
        );
    }

    #[test]
    fn validate_duplicate_alias() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.set_alias(NetRef::InstancePort("u_a".into(), "clk".into()), "sys_clk");
        s.set_alias(NetRef::InstancePort("u_b".into(), "clk".into()), "sys_clk");

        let diags = s.validate(&[]);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("alias 'sys_clk'"))
        );
    }

    #[test]
    fn validate_unconnected_warning() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "fifo_sync").unwrap();
        // Don't connect any ports

        let lib = vec![make_module(
            "fifo_sync",
            vec![
                make_port("clk", Direction::In, PortType::StdLogic),
                make_port("data", Direction::Out, PortType::StdLogic),
            ],
        )];
        let diags = s.validate(&lib);
        // Should have warnings, not errors
        assert!(diags.iter().all(|d| !d.is_error()));
        assert!(diags.iter().any(|d| d.message.contains("unconnected")));
    }

    #[test]
    fn validate_unresolved_module() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_fifo", "nonexistent_module").unwrap();

        let diags = s.validate(&[]);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("not found in library"))
        );
    }

    #[test]
    fn remove_instance_sweeps_sibling_port_maps() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();
        s.set_port_map_entry(
            "u_b",
            "clk",
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "dout".into(),
                SliceExpr::Bit(0),
            )),
        )
        .unwrap();

        s.remove_instance("u_a").unwrap();

        let u_b = s.get_instance("u_b").unwrap();
        assert_eq!(u_b.port_map.get("din"), Some(&None));
        assert_eq!(u_b.port_map.get("clk"), Some(&None));
    }

    #[test]
    fn remove_instance_drops_aliases() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_pll", "pll").unwrap();
        s.set_alias(
            NetRef::InstancePort("u_pll".into(), "clk_out".into()),
            "sys_clk",
        );
        s.remove_instance("u_pll").unwrap();
        assert!(s.aliases.is_empty());
    }

    #[test]
    fn multi_load_net_passes_validation() {
        // Three instance inputs all driven by the same top-level clock.
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(PortDef {
            name: "clk".into(),
            direction: Direction::In,
            port_type: PortType::StdLogic,
            bundle: None,
        });
        for n in ["u_a", "u_b", "u_c"] {
            s.add_instance(n, "mod_x").unwrap();
            s.set_port_map_entry(n, "clk", Some(NetRef::TopPort("clk".into())))
                .unwrap();
        }
        let lib = vec![make_module(
            "mod_x",
            vec![make_port("clk", Direction::In, PortType::StdLogic)],
        )];
        let diags = s.validate(&lib);
        // No errors — only warnings are allowed (e.g. unconnected-port noise for
        // other ports, of which this module has none).
        assert!(
            diags.iter().all(|d| !d.is_error()),
            "expected no errors, got {:?}",
            diags
        );
    }

    #[test]
    fn validate_scalar_vs_unresolved_vector_mismatch() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "clk",
            Some(NetRef::InstancePort("u_a".into(), "dout".into())),
        )
        .unwrap();

        let lib = vec![
            make_module(
                "mod_a",
                vec![make_port(
                    "dout",
                    Direction::Out,
                    PortType::StdLogicVector(Range {
                        high: RangeExpr::Expr("WIDTH-1".into()),
                        low: RangeExpr::Literal(0),
                        dir: RangeDir::Downto,
                    }),
                )],
            ),
            make_module(
                "mod_b",
                vec![make_port("clk", Direction::In, PortType::StdLogic)],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("type mismatch")),
            "{:?}",
            diags
        );
    }

    #[test]
    fn manual_bundle_default_empty() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        let inst = s.get_instance("u_a").unwrap();
        assert!(inst.manual_bundles.is_empty());
    }

    #[test]
    fn validate_manual_bundle_unknown_port() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.get_instance_mut("u_a")
            .unwrap()
            .manual_bundles
            .insert("spi".into(), vec!["mosi".into(), "bogus_port".into()]);
        let lib = vec![make_module(
            "mod_a",
            vec![make_port("mosi", Direction::In, PortType::StdLogic)],
        )];
        let diags = s.validate(&lib);
        assert!(
            diags.iter().any(|d| d.is_error()
                && d.message.contains("bogus_port")
                && d.message.contains("manual bundle")),
            "{:?}",
            diags
        );
    }

    #[test]
    fn validate_slice_out_of_range() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.add_instance("u_a", "mod_a").unwrap();
        s.add_instance("u_b", "mod_b").unwrap();
        s.set_port_map_entry(
            "u_b",
            "din",
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "dout".into(),
                SliceExpr::Range { high: 15, low: 8 },
            )),
        )
        .unwrap();

        let lib = vec![
            make_module(
                "mod_a",
                vec![make_port(
                    "dout",
                    Direction::Out,
                    PortType::StdLogicVector(Range {
                        high: RangeExpr::Literal(7),
                        low: RangeExpr::Literal(0),
                        dir: RangeDir::Downto,
                    }),
                )],
            ),
            make_module(
                "mod_b",
                vec![make_port(
                    "din",
                    Direction::In,
                    PortType::StdLogicVector(Range {
                        high: RangeExpr::Literal(7),
                        low: RangeExpr::Literal(0),
                        dir: RangeDir::Downto,
                    }),
                )],
            ),
        ];
        let diags = s.validate(&lib);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("out of range")),
            "{:?}",
            diags
        );
    }

    #[test]
    fn replace_top_level_clears_dropped_port_refs() {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports
            .push(make_port("clk", Direction::In, PortType::StdLogic));
        s.top_ports
            .push(make_port("rst", Direction::In, PortType::StdLogic));
        s.add_instance("u_a", "mod_a").unwrap();
        s.set_port_map_entry("u_a", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        s.set_port_map_entry("u_a", "rst_n", Some(NetRef::TopPort("rst".into())))
            .unwrap();
        // Drop the `rst` top port. `clk` is preserved.
        let removed = s.replace_top_level(
            "top".into(),
            Vec::new(),
            vec![make_port("clk", Direction::In, PortType::StdLogic)],
        );
        assert_eq!(removed, vec!["rst".to_string()]);
        let inst = s.get_instance("u_a").unwrap();
        assert_eq!(
            inst.port_map.get("clk"),
            Some(&Some(NetRef::TopPort("clk".into())))
        );
        assert_eq!(inst.port_map.get("rst_n"), Some(&None));
    }

    // --- Regressions: defects in a hand-written project that `validate`
    // accepted with "No errors" ---

    /// `led_driver` from the `--example` project, wired to top-level ports.
    fn led_driver_project() -> (Schematic, Vec<ModuleDef>) {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports
            .push(make_port("clk", Direction::In, PortType::StdLogic));
        s.top_ports
            .push(make_port("led", Direction::Out, PortType::StdLogic));
        s.add_instance("u_a", "led_driver").unwrap();
        s.set_port_map_entry("u_a", "clk", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        s.set_port_map_entry("u_a", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let lib = vec![make_module(
            "led_driver",
            vec![
                make_port("clk", Direction::In, PortType::StdLogic),
                make_port("led", Direction::Out, PortType::StdLogic),
            ],
        )];
        (s, lib)
    }

    #[test]
    fn regression_project_is_clean_without_a_defect() {
        let (s, lib) = led_driver_project();
        let diags = s.validate(&lib);
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn port_map_key_that_is_not_a_port_is_an_error() {
        let (mut s, lib) = led_driver_project();
        s.set_port_map_entry("u_a", "bogus", Some(NetRef::TopPort("clk".into())))
            .unwrap();
        let diags = s.validate(&lib);
        assert!(
            diags.iter().any(|d| d.is_error()
                && d.message.contains("'bogus'")
                && d.instance.as_deref() == Some("u_a")),
            "{diags:?}"
        );
    }

    #[test]
    fn generic_map_key_that_is_not_a_generic_is_an_error() {
        let (mut s, lib) = led_driver_project();
        s.set_generic_map_entry("u_a", "NOPE", "1").unwrap();
        let diags = s.validate(&lib);
        assert!(
            diags.iter().any(|d| d.is_error()
                && d.message.contains("'NOPE'")
                && d.instance.as_deref() == Some("u_a")),
            "{diags:?}"
        );
    }

    #[test]
    fn top_port_without_a_connection_is_a_warning() {
        let (mut s, lib) = led_driver_project();
        s.top_ports
            .push(make_port("spare", Direction::Out, PortType::StdLogic));
        let diags = s.validate(&lib);
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(!diags[0].is_error());
        assert!(diags[0].message.contains("'spare'"), "{diags:?}");
    }

    /// `pulse_gen` from the `--example` project: the width of `count`
    /// depends on the generic `WIDTH`, which has the default value 8.
    fn pulse_gen_on_a_16_bit_top_port(width: Option<&str>) -> Vec<Diagnostic> {
        let vector = |high: RangeExpr| {
            PortType::StdLogicVector(Range {
                high,
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            })
        };
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(make_port(
            "wide",
            Direction::Out,
            vector(RangeExpr::Literal(15)),
        ));
        s.add_instance("u_gen", "pulse_gen").unwrap();
        s.set_port_map_entry("u_gen", "count", Some(NetRef::TopPort("wide".into())))
            .unwrap();
        if let Some(width) = width {
            s.set_generic_map_entry("u_gen", "WIDTH", width).unwrap();
        }
        let mut module = make_module(
            "pulse_gen",
            vec![make_port(
                "count",
                Direction::Out,
                vector(RangeExpr::Expr("WIDTH - 1".into())),
            )],
        );
        module.generics.push(GenericDef {
            name: "WIDTH".into(),
            type_name: "integer".into(),
            default_value: Some("8".into()),
        });
        s.validate(&[module])
    }

    #[test]
    fn width_from_a_generic_default_is_checked() {
        let diags = pulse_gen_on_a_16_bit_top_port(None);
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("width mismatch")),
            "{diags:?}"
        );
    }

    #[test]
    fn width_from_a_generic_map_value_is_checked() {
        let diags = pulse_gen_on_a_16_bit_top_port(Some("16"));
        assert!(diags.is_empty(), "{diags:?}");
        let diags = pulse_gen_on_a_16_bit_top_port(Some("12"));
        assert!(
            diags
                .iter()
                .any(|d| d.is_error() && d.message.contains("width mismatch")),
            "{diags:?}"
        );
    }

    /// The 29-bit output `count` with a consumer slice, on the top-level
    /// port `led`. This is the project of the codegen test
    /// `sliced_wide_driver_intermediate_uses_port_width`.
    fn consumer_slice_on_a_top_port(slice: SliceExpr, led: PortType) -> Vec<String> {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(make_port("led", Direction::Out, led));
        s.add_instance("u_pulse", "pulse_gen").unwrap();
        s.set_port_map_entry("u_pulse", "count", Some(NetRef::TopPort("led".into())))
            .unwrap();
        s.instances[0].consumer_slices.insert("count".into(), slice);
        let lib = vec![make_module(
            "pulse_gen",
            vec![make_port("count", Direction::Out, literal_vector(28))],
        )];
        s.validate(&lib)
            .into_iter()
            .filter(|d| d.is_error())
            .map(|d| d.message)
            .collect()
    }

    fn literal_vector(high: i64) -> PortType {
        PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(high),
            low: RangeExpr::Literal(0),
            dir: RangeDir::Downto,
        })
    }

    #[test]
    fn consumer_slice_sets_the_width_of_the_pin() {
        let five_bits = SliceExpr::Range { high: 28, low: 24 };
        // Regression: "width mismatch: port is 29 bits but driver 'led' is
        // 5 bits" for five bits of the port on a 5-bit top-level port.
        let errors = consumer_slice_on_a_top_port(five_bits.clone(), literal_vector(4));
        assert!(errors.is_empty(), "{errors:?}");

        // Regression: one bit of the port on a scalar was a mismatch.
        let errors = consumer_slice_on_a_top_port(SliceExpr::Bit(3), PortType::StdLogic);
        assert!(errors.is_empty(), "{errors:?}");

        // The five bits do not fit an 8-bit top-level port.
        let errors = consumer_slice_on_a_top_port(five_bits, literal_vector(7));
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("port is 5 bits"), "{errors:?}");
    }

    fn integer_generic(name: &str, default_value: Option<&str>) -> GenericDef {
        GenericDef {
            name: name.into(),
            type_name: "integer".into(),
            default_value: default_value.map(Into::into),
        }
    }

    /// `pulse_gen` with `count : out std_logic_vector(WIDTH - 1 downto 0)`.
    fn pulse_gen_module(width_default: &str) -> ModuleDef {
        let mut module = make_module(
            "pulse_gen",
            vec![make_port(
                "count",
                Direction::Out,
                PortType::StdLogicVector(Range {
                    high: RangeExpr::Expr("WIDTH - 1".into()),
                    low: RangeExpr::Literal(0),
                    dir: RangeDir::Downto,
                }),
            )],
        );
        module
            .generics
            .push(integer_generic("WIDTH", Some(width_default)));
        module
    }

    /// `u_gen.count` gets its width from the top-level generic `TOP_W`
    /// (`WIDTH => <width_value>`), and drives the 8-bit input `u_led.value`.
    fn top_generic_width_on_an_8_bit_input(
        width_value: &str,
        top_default: Option<&str>,
    ) -> Vec<Diagnostic> {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_generics.push(integer_generic("TOP_W", top_default));
        s.top_ports
            .push(make_port("led", Direction::Out, PortType::StdLogic));
        s.add_instance("u_gen", "pulse_gen").unwrap();
        s.set_generic_map_entry("u_gen", "WIDTH", width_value)
            .unwrap();
        s.add_instance("u_led", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_led",
            "value",
            Some(NetRef::InstancePort("u_gen".into(), "count".into())),
        )
        .unwrap();
        s.set_port_map_entry("u_led", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        let lib = vec![
            pulse_gen_module("8"),
            make_module(
                "led_driver",
                vec![
                    make_port("value", Direction::In, literal_vector(7)),
                    make_port("led", Direction::Out, PortType::StdLogic),
                ],
            ),
        ];
        s.validate(&lib)
    }

    /// Regression: a width from a top-level generic was unknown, so a
    /// 16-bit output on an 8-bit input passed. The HDL tool elaborates the
    /// top level with the default values, and rejects it.
    #[test]
    fn width_from_a_top_generic_is_checked_at_its_default_value() {
        let diags = top_generic_width_on_an_8_bit_input("TOP_W", Some("16"));
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(diags[0].is_error());
        let message = &diags[0].message;
        assert!(
            message.contains("port is 8 bits") && message.contains("is 16 bits"),
            "{message}"
        );
        assert!(message.contains("default value"), "{message}");

        let diags = top_generic_width_on_an_8_bit_input("TOP_W", Some("8"));
        assert!(diags.is_empty(), "{diags:?}");
    }

    /// Regression: a width from an expression, such as `2 * TOP_W`, had no
    /// value, so the check made no claim for it.
    #[test]
    fn width_from_an_expression_of_a_top_generic_is_checked() {
        let diags = top_generic_width_on_an_8_bit_input("2 * TOP_W", Some("8"));
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(
            diags[0].message.contains("is 16 bits"),
            "{}",
            diags[0].message
        );
        let diags = top_generic_width_on_an_8_bit_input("2 * TOP_W", Some("4"));
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn width_from_a_top_generic_with_no_default_is_not_checked() {
        let diags = top_generic_width_on_an_8_bit_input("TOP_W", None);
        assert!(diags.is_empty(), "{diags:?}");
    }

    /// `u_b.ena` (one bit) takes the bit `bit` of the pin `u_a.count`. The
    /// port `count` has 32 bits before `configure` changes the instance.
    fn bit_of_a_pin(bit: i32, configure: impl FnOnce(&mut Instance)) -> Vec<String> {
        let mut s = Schematic::new("top", Language::Vhdl);
        configure(s.add_instance("u_a", "pulse_gen").unwrap());
        s.add_instance("u_b", "gate").unwrap();
        s.set_port_map_entry(
            "u_b",
            "ena",
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "count".into(),
                SliceExpr::Bit(bit),
            )),
        )
        .unwrap();
        let lib = vec![
            pulse_gen_module("32"),
            make_module(
                "gate",
                vec![make_port("ena", Direction::In, PortType::StdLogic)],
            ),
        ];
        s.validate(&lib)
            .into_iter()
            .filter(|d| d.is_error())
            .map(|d| d.message)
            .collect()
    }

    /// Regression: the range check used the port as written, so it made no
    /// check for a width from a generic.
    #[test]
    fn slice_range_uses_the_width_from_the_generic_map() {
        let four_bits = |inst: &mut Instance| {
            inst.generic_map.insert("WIDTH".into(), "4".into());
        };
        let errors = bit_of_a_pin(7, four_bits);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("out of range for driver of width 4"));
        let errors = bit_of_a_pin(3, four_bits);
        assert!(errors.is_empty(), "{errors:?}");
    }

    /// Regression: the range check used the 32 bits of the port for a pin
    /// that connects only five of them.
    #[test]
    fn slice_range_uses_the_width_of_the_consumer_slice() {
        let five_bits = |inst: &mut Instance| {
            inst.consumer_slices
                .insert("count".into(), SliceExpr::Range { high: 28, low: 24 });
        };
        let errors = bit_of_a_pin(10, five_bits);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("out of range for driver of width 5"));
        let errors = bit_of_a_pin(4, five_bits);
        assert!(errors.is_empty(), "{errors:?}");
    }

    /// An 8-bit output on a slice of the 16-bit top-level port `wide`.
    fn eight_bits_on_a_slice(slice: SliceExpr, consumer_slice: Option<SliceExpr>) -> Vec<String> {
        let vector = |high: i64| {
            PortType::StdLogicVector(Range {
                high: RangeExpr::Literal(high),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            })
        };
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports
            .push(make_port("wide", Direction::Out, vector(15)));
        s.add_instance("u_lo", "byte_source").unwrap();
        s.set_port_map_entry(
            "u_lo",
            "count",
            Some(NetRef::TopPortSlice("wide".into(), slice)),
        )
        .unwrap();
        if let Some(consumer_slice) = consumer_slice {
            s.instances[0]
                .consumer_slices
                .insert("count".into(), consumer_slice);
        }
        let lib = vec![make_module(
            "byte_source",
            vec![make_port("count", Direction::Out, vector(7))],
        )];
        s.validate(&lib)
            .into_iter()
            .filter(|d| d.is_error())
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn slice_reference_width_is_checked() {
        let errors = eight_bits_on_a_slice(SliceExpr::Range { high: 7, low: 0 }, None);
        assert!(errors.is_empty(), "{errors:?}");

        let errors = eight_bits_on_a_slice(SliceExpr::Range { high: 3, low: 0 }, None);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("width mismatch"), "{errors:?}");

        // Four bits of the port on four bits of the net.
        let errors = eight_bits_on_a_slice(
            SliceExpr::Range { high: 3, low: 0 },
            Some(SliceExpr::Range { high: 7, low: 4 }),
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    /// A generic that the instance sets to a name (a top-level generic) has
    /// no known value. The check must not use the default value in its place.
    #[test]
    fn width_from_a_generic_with_an_unknown_value_is_not_checked() {
        let diags = pulse_gen_on_a_16_bit_top_port(Some("TOP_WIDTH"));
        assert!(diags.is_empty(), "{diags:?}");
    }
}

#[cfg(test)]
mod slice_driver_tests {
    use super::*;

    fn led_module() -> ModuleDef {
        ModuleDef {
            name: "led_driver".to_string(),
            generics: Vec::new(),
            ports: vec![PortDef {
                name: "led".to_string(),
                direction: Direction::Out,
                port_type: PortType::StdLogic,
                bundle: None,
            }],
            source_path: "led_driver.vhd".into(),
            source_hash: 0,
            dependencies: Vec::new(),
        }
    }

    fn vector_top(width_high: i64) -> Schematic {
        let mut s = Schematic::new("top", Language::Vhdl);
        s.top_ports.push(PortDef {
            name: "led".to_string(),
            direction: Direction::Out,
            port_type: PortType::StdLogicVector(Range {
                high: RangeExpr::Literal(width_high),
                low: RangeExpr::Literal(0),
                dir: RangeDir::Downto,
            }),
            bundle: None,
        });
        s
    }

    #[test]
    fn disjoint_bit_drivers_are_not_multi_driver() {
        let mut s = vector_top(4);
        let lib = vec![led_module()];
        for b in 0..5 {
            s.add_instance(format!("u{b}"), "led_driver").unwrap();
            s.set_port_map_entry(
                &format!("u{b}"),
                "led",
                Some(NetRef::TopPortSlice("led".into(), SliceExpr::Bit(b))),
            )
            .unwrap();
        }
        let errors: Vec<_> = s
            .validate(&lib)
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn overlapping_bit_drivers_still_error() {
        let mut s = vector_top(4);
        let lib = vec![led_module()];
        for name in ["u0", "u1"] {
            s.add_instance(name, "led_driver").unwrap();
            s.set_port_map_entry(
                name,
                "led",
                Some(NetRef::TopPortSlice("led".into(), SliceExpr::Bit(2))),
            )
            .unwrap();
        }
        let errors: Vec<_> = s
            .validate(&lib)
            .into_iter()
            .filter(|d| d.is_error() && d.message.contains("multiple drivers"))
            .collect();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn full_port_driver_conflicts_with_bit_driver() {
        let mut s = vector_top(4);
        let lib = vec![led_module()];
        s.add_instance("u_full", "led_driver").unwrap();
        s.set_port_map_entry("u_full", "led", Some(NetRef::TopPort("led".into())))
            .unwrap();
        s.add_instance("u_bit", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_bit",
            "led",
            Some(NetRef::TopPortSlice("led".into(), SliceExpr::Bit(0))),
        )
        .unwrap();
        let errors: Vec<_> = s
            .validate(&lib)
            .into_iter()
            .filter(|d| d.is_error() && d.message.contains("multiple drivers"))
            .collect();
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn overlapping_range_drivers_error() {
        let mut s = vector_top(7);
        let lib = vec![led_module()];
        s.add_instance("u_lo", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_lo",
            "led",
            Some(NetRef::TopPortSlice(
                "led".into(),
                SliceExpr::Range { high: 4, low: 0 },
            )),
        )
        .unwrap();
        s.add_instance("u_hi", "led_driver").unwrap();
        s.set_port_map_entry(
            "u_hi",
            "led",
            Some(NetRef::TopPortSlice(
                "led".into(),
                SliceExpr::Range { high: 7, low: 4 },
            )),
        )
        .unwrap();
        let errors: Vec<_> = s
            .validate(&lib)
            .into_iter()
            .filter(|d| d.is_error() && d.message.contains("multiple drivers"))
            .collect();
        assert_eq!(errors.len(), 1, "bit 4 overlaps: {errors:?}");
    }
}
