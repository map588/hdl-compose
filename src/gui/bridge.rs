#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qproperty(QString, project_name)]
        #[qproperty(bool, dirty)]
        type AppState = super::AppStateRust;

        #[qinvokable]
        fn open_project(self: Pin<&mut AppState>, path: &QString) -> bool;

        #[qinvokable]
        fn save_project(self: Pin<&mut AppState>) -> bool;

        #[qinvokable]
        fn save_project_as(self: Pin<&mut AppState>, path: &QString) -> bool;

        #[qinvokable]
        fn generate_code(self: Pin<&mut AppState>, path: &QString) -> bool;

        #[qinvokable]
        fn suggest_codegen_path(self: &AppState) -> QString;

        #[qinvokable]
        fn project_language(self: &AppState) -> i32;

        #[qinvokable]
        fn top_level_buffer(self: &AppState) -> QString;

        #[qinvokable]
        fn commit_top_level_buffer(self: Pin<&mut AppState>, buffer: &QString) -> bool;

        #[qinvokable]
        fn new_project(self: Pin<&mut AppState>, name: &QString, language: i32) -> bool;

        #[qinvokable]
        fn has_project(self: &AppState) -> bool;

        #[qinvokable]
        fn last_error(self: &AppState) -> QString;

        #[qinvokable]
        fn instance_count(self: &AppState) -> i32;

        #[qinvokable]
        fn instance_name(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn instance_module(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn add_instance(self: Pin<&mut AppState>, name: &QString, module: &QString) -> bool;

        #[qinvokable]
        fn remove_instance(self: Pin<&mut AppState>, name: &QString) -> bool;

        #[qinvokable]
        fn rename_instance(
            self: Pin<&mut AppState>,
            old_name: &QString,
            new_name: &QString,
        ) -> bool;

        #[qinvokable]
        fn instance_is_dirty(self: &AppState, index: i32) -> bool;

        /// Index of an instance by name, -1 if absent. One FFI call instead
        /// of an instance_name(i) loop from C++.
        #[qinvokable]
        fn instance_index(self: &AppState, name: &QString) -> i32;

        /// Dirty flag by name — paint-path helper (called every frame).
        #[qinvokable]
        fn instance_is_dirty_name(self: &AppState, name: &QString) -> bool;

        #[qinvokable]
        fn instance_source_path(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn set_instance_position(
            self: Pin<&mut AppState>,
            name: &QString,
            x: f64,
            y: f64,
        ) -> bool;

        #[qinvokable]
        fn instance_pos_x(self: &AppState, index: i32) -> f64;

        #[qinvokable]
        fn instance_pos_y(self: &AppState, index: i32) -> f64;

        #[qinvokable]
        fn set_selected_instance(self: Pin<&mut AppState>, name: &QString);

        #[qinvokable]
        fn selected_instance(self: &AppState) -> QString;

        /// Group multiple mutations into one undo step, one validation pass,
        /// and one port_map_changed_bulk signal. Nestable; only the
        /// outermost pair snapshots and signals.
        #[qinvokable]
        fn begin_batch(self: Pin<&mut AppState>);

        #[qinvokable]
        fn end_batch(self: Pin<&mut AppState>);

        #[qinvokable]
        fn set_port_map_entry(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
            rhs: &QString,
        ) -> bool;

        #[qinvokable]
        fn set_port_map_entry_slice(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
            driver_inst: &QString,
            driver_port: &QString,
            slice_high: i32,
            slice_low: i32,
        ) -> bool;

        #[qinvokable]
        fn set_consumer_slice(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
            slice_high: i32,
            slice_low: i32,
        ) -> bool;

        #[qinvokable]
        fn clear_consumer_slice(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
        ) -> bool;

        #[qinvokable]
        fn consumer_slice(
            self: &AppState,
            instance: &QString,
            port: &QString,
        ) -> QString;

        #[qinvokable]
        fn clear_port_map_entry(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
        ) -> bool;

        #[qinvokable]
        fn port_map_entry(
            self: &AppState,
            instance: &QString,
            port: &QString,
        ) -> QString;

        #[qinvokable]
        fn reload_library(self: Pin<&mut AppState>) -> bool;

        #[qinvokable]
        fn match_by_name(self: Pin<&mut AppState>, instance: &QString) -> i32;

        #[qinvokable]
        fn clear_instance_dirty(self: Pin<&mut AppState>, name: &QString) -> bool;

        #[qinvokable]
        fn undo(self: Pin<&mut AppState>) -> bool;

        #[qinvokable]
        fn redo(self: Pin<&mut AppState>) -> bool;

        #[qinvokable]
        fn can_undo(self: &AppState) -> bool;

        #[qinvokable]
        fn can_redo(self: &AppState) -> bool;

        #[qinvokable]
        fn module_generic_count(self: &AppState, instance_index: i32) -> i32;

        #[qinvokable]
        fn module_generic_name(
            self: &AppState,
            instance_index: i32,
            generic_index: i32,
        ) -> QString;

        #[qinvokable]
        fn module_generic_default(
            self: &AppState,
            instance_index: i32,
            generic_index: i32,
        ) -> QString;

        #[qinvokable]
        fn generic_map_entry(
            self: &AppState,
            instance: &QString,
            generic_name: &QString,
        ) -> QString;

        #[qinvokable]
        fn create_manual_bundle(
            self: Pin<&mut AppState>,
            instance: &QString,
            name: &QString,
            ports_csv: &QString,
        ) -> bool;

        #[qinvokable]
        fn remove_manual_bundle(
            self: Pin<&mut AppState>,
            instance: &QString,
            name: &QString,
        ) -> bool;

        #[qinvokable]
        fn manual_bundle_count(self: &AppState, instance_index: i32) -> i32;

        #[qinvokable]
        fn manual_bundle_name(
            self: &AppState,
            instance_index: i32,
            bundle_index: i32,
        ) -> QString;

        #[qinvokable]
        fn manual_bundle_port_count(
            self: &AppState,
            instance_index: i32,
            bundle_index: i32,
        ) -> i32;

        #[qinvokable]
        fn manual_bundle_port_name(
            self: &AppState,
            instance_index: i32,
            bundle_index: i32,
            port_index: i32,
        ) -> QString;

        #[qinvokable]
        fn promote_port_to_top(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
        ) -> QString;

        #[qinvokable]
        fn set_generic_map_entry(
            self: Pin<&mut AppState>,
            instance: &QString,
            generic: &QString,
            expr: &QString,
        ) -> bool;

        #[qinvokable]
        fn instance_port_count(self: &AppState, instance_index: i32) -> i32;

        #[qinvokable]
        fn instance_port_name(
            self: &AppState,
            instance_index: i32,
            port_index: i32,
        ) -> QString;

        #[qinvokable]
        fn instance_port_direction(
            self: &AppState,
            instance_index: i32,
            port_index: i32,
        ) -> i32;

        #[qinvokable]
        fn instance_port_width(
            self: &AppState,
            instance_index: i32,
            port_index: i32,
        ) -> i32;

        #[qinvokable]
        fn instance_port_bundle(
            self: &AppState,
            instance_index: i32,
            port_index: i32,
        ) -> QString;

        #[qinvokable]
        fn instance_dependency_count(self: &AppState, instance_index: i32) -> i32;

        #[qinvokable]
        fn instance_dependency_name(
            self: &AppState,
            instance_index: i32,
            dep_index: i32,
        ) -> QString;

        #[qinvokable]
        fn instance_dependency_present(
            self: &AppState,
            instance_index: i32,
            dep_index: i32,
        ) -> bool;

        #[qinvokable]
        fn top_port_count(self: &AppState) -> i32;

        #[qinvokable]
        fn top_port_name(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn top_port_direction(self: &AppState, index: i32) -> i32;

        #[qinvokable]
        fn top_port_width(self: &AppState, index: i32) -> i32;

        #[qinvokable]
        fn wire_count(self: &AppState) -> i32;

        #[qinvokable]
        fn wire_source(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn wire_target(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn wire_width(self: &AppState, index: i32) -> i32;

        #[qinvokable]
        fn set_alias(
            self: Pin<&mut AppState>,
            net_key: &QString,
            alias: &QString,
        ) -> bool;

        #[qinvokable]
        fn library_module_count(self: &AppState) -> i32;

        #[qinvokable]
        fn library_module_name(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn add_library_path(self: Pin<&mut AppState>, path: &QString) -> bool;

        #[qinvokable]
        fn library_path_count(self: &AppState) -> i32;

        #[qinvokable]
        fn library_path(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn remove_library_path(self: Pin<&mut AppState>, path: &QString) -> bool;

        #[qinvokable]
        fn current_project_path(self: &AppState) -> QString;

        #[qinvokable]
        fn validation_count(self: &AppState) -> i32;

        #[qinvokable]
        fn validation_error_count(self: &AppState) -> i32;

        #[qinvokable]
        fn validation_warning_count(self: &AppState) -> i32;

        #[qinvokable]
        fn validation_message(self: &AppState, index: i32) -> QString;

        #[qsignal]
        fn project_loaded(self: Pin<&mut AppState>);

        #[qsignal]
        fn project_saved(self: Pin<&mut AppState>);

        #[qsignal]
        fn instance_added(self: Pin<&mut AppState>, name: QString);

        #[qsignal]
        fn instance_removed(self: Pin<&mut AppState>, name: QString);

        #[qsignal]
        fn port_map_changed(self: Pin<&mut AppState>, instance: QString, port: QString);

        #[qsignal]
        fn port_map_changed_bulk(self: Pin<&mut AppState>);

        #[qsignal]
        fn alias_changed(self: Pin<&mut AppState>, key: QString);

        #[qsignal]
        fn validation_changed(self: Pin<&mut AppState>);

        #[qsignal]
        fn instance_moved(self: Pin<&mut AppState>, name: QString, x: f64, y: f64);

        #[qsignal]
        fn selection_changed(self: Pin<&mut AppState>, name: QString);

        #[qsignal]
        fn library_changed(self: Pin<&mut AppState>);
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::SystemTime;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

use crate::codegen;
use crate::project;
use crate::schematic::Diagnostic;
use crate::types::{
    Direction, Language, ModuleDef, NetRef, PortDef, PortType, Range, RangeDir, RangeExpr,
    Schematic, SliceExpr,
};

#[derive(Default)]
pub struct AppStateRust {
    // Qt properties (auto getters/setters/signals)
    project_name: QString,
    dirty: bool,
    // Internal state — not directly exposed as properties
    schematic: Option<Schematic>,
    current_path: Option<PathBuf>,
    last_error: String,
    library: Vec<ModuleDef>,
    diagnostics: Vec<Diagnostic>,
    selected_instance: String,
    // Flattened wire list cache: (driver_key, target_key, width).
    // driver_key: "top:<name>" or "<inst>.<port>" (with optional `[h:l]` slice)
    // target_key: always "<inst>.<port>"
    // width: bus width of the wire (1 for scalar, N for vector); -1 unknown.
    wires: Vec<(String, String, i32)>,
    // Undo/redo: JSON snapshots of `schematic` taken before each mutation.
    // Capped at UNDO_STACK_LIMIT entries.
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    // Parsed-module cache keyed by source path, invalidated when the file's
    // mtime changes. Without it every mutation re-parses the whole library
    // from disk (rebuild_library_and_validate runs per mutation).
    parse_cache: HashMap<PathBuf, ParseCacheEntry>,
    // Mutation batching: depth > 0 while inside begin_batch/end_batch.
    // Batched mutators skip per-entry snapshot/validate/signal; end_batch
    // does each once.
    batch_depth: u32,
    batch_changed: bool,
}

struct ParseCacheEntry {
    mtime: SystemTime,
    modules: Vec<ModuleDef>,
    error: Option<String>,
}

// Resolve library modules through the parse cache. Entries re-parse only
// when the file's mtime moved; vanished paths fall out of the cache.
fn resolve_library_cached(
    cache: &mut HashMap<PathBuf, ParseCacheEntry>,
    paths: &[PathBuf],
) -> (Vec<ModuleDef>, Vec<(PathBuf, String)>) {
    cache.retain(|k, _| paths.contains(k));
    let mut modules = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        if let (Some(mt), Some(entry)) = (mtime, cache.get(path))
            && entry.mtime == mt
        {
            modules.extend(entry.modules.iter().cloned());
            if let Some(e) = &entry.error {
                errors.push((path.clone(), e.clone()));
            }
            continue;
        }
        match crate::parse_file(path) {
            Ok(defs) => {
                if let Some(mt) = mtime {
                    cache.insert(
                        path.clone(),
                        ParseCacheEntry { mtime: mt, modules: defs.clone(), error: None },
                    );
                }
                modules.extend(defs);
            }
            Err(e) => {
                let msg = e.to_string();
                if let Some(mt) = mtime {
                    cache.insert(
                        path.clone(),
                        ParseCacheEntry { mtime: mt, modules: Vec::new(), error: Some(msg.clone()) },
                    );
                }
                errors.push((path.clone(), msg));
            }
        }
    }
    (modules, errors)
}

const UNDO_STACK_LIMIT: usize = 100;

impl qobject::AppState {
    fn record_error(mut self: Pin<&mut Self>, msg: impl Into<String>) {
        self.as_mut().rust_mut().get_mut().last_error = msg.into();
    }

    /// Snapshot for undo unless inside a batch (begin_batch already took one).
    fn snapshot_for_mutation(mut self: Pin<&mut Self>) {
        if self.as_ref().rust().batch_depth == 0 {
            self.as_mut().push_snapshot();
        }
    }

    /// Post-mutation bookkeeping. Inside a batch just mark changed —
    /// end_batch validates and signals once. Outside, validate + signal now:
    /// `Some((inst, port))` emits port_map_changed, `None` emits
    /// port_map_changed_bulk.
    fn finish_mutation(mut self: Pin<&mut Self>, entry: Option<(&str, &str)>) {
        {
            let m = self.as_mut().rust_mut().get_mut();
            m.dirty = true;
            if m.batch_depth > 0 {
                m.batch_changed = true;
                return;
            }
        }
        self.as_mut().set_dirty(true);
        // Refresh wire cache BEFORE firing the signal — canvas reads the
        // cache in its port_map_changed handler.
        self.as_mut().rebuild_library_and_validate();
        match entry {
            Some((inst, port)) => {
                self.as_mut()
                    .port_map_changed(QString::from(inst), QString::from(port));
            }
            None => self.as_mut().port_map_changed_bulk(),
        }
    }

    pub fn begin_batch(mut self: Pin<&mut Self>) {
        if self.as_ref().rust().batch_depth == 0 {
            self.as_mut().push_snapshot();
        }
        self.as_mut().rust_mut().get_mut().batch_depth += 1;
    }

    pub fn end_batch(mut self: Pin<&mut Self>) {
        let changed = {
            let m = self.as_mut().rust_mut().get_mut();
            if m.batch_depth == 0 {
                return;
            }
            m.batch_depth -= 1;
            if m.batch_depth > 0 {
                return;
            }
            let c = m.batch_changed;
            m.batch_changed = false;
            c
        };
        if changed {
            self.as_mut().set_dirty(true);
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().port_map_changed_bulk();
        }
    }

    fn instance_module_ports(&self, instance_index: i32) -> Option<&[PortDef]> {
        let r = self.rust();
        let s = r.schematic.as_ref()?;
        let inst = s.instances.get(instance_index as usize)?;
        let module = r.library.iter().find(|m| m.name == inst.module_ref)?;
        Some(&module.ports)
    }

    fn instance_module_def(&self, instance_index: i32) -> Option<&ModuleDef> {
        let r = self.rust();
        let s = r.schematic.as_ref()?;
        let inst = s.instances.get(instance_index as usize)?;
        r.library.iter().find(|m| m.name == inst.module_ref)
    }

    fn instance_port_at(&self, instance_index: i32, port_index: i32) -> Option<&PortDef> {
        self.instance_module_ports(instance_index)
            .and_then(|ports| ports.get(port_index as usize))
    }

    fn top_port_at(&self, index: i32) -> Option<&PortDef> {
        self.rust()
            .schematic
            .as_ref()?
            .top_ports
            .get(index as usize)
    }

    fn refresh_project_display(mut self: Pin<&mut Self>) {
        let (name, dirty) = match &self.as_ref().rust().schematic {
            Some(s) => (QString::from(&s.top_name), self.as_ref().rust().dirty),
            None => (QString::default(), false),
        };
        self.as_mut().set_project_name(name);
        self.as_mut().set_dirty(dirty);
    }

    fn rebuild_library_and_validate(mut self: Pin<&mut Self>) {
        let paths: Vec<PathBuf> = match self.as_ref().rust().schematic.as_ref() {
            Some(s) => s.library_paths.clone(),
            None => return,
        };
        let (library, lib_errors) = {
            let cache = &mut self.as_mut().rust_mut().get_mut().parse_cache;
            resolve_library_cached(cache, &paths)
        };
        let mut diagnostics = match self.as_ref().rust().schematic.as_ref() {
            Some(s) => s.validate(&library),
            None => return,
        };
        for (path, err) in lib_errors {
            diagnostics.push(Diagnostic::error(format!(
                "library: {}: {err}",
                path.display()
            )));
        }
        // Surface the most recent library error to the status bar.
        if let Some(d) = diagnostics
            .iter()
            .find(|d| d.is_error() && d.message.starts_with("library: "))
        {
            let msg = d.message.clone();
            self.as_mut().record_error(msg);
        }
        {
            let m = self.as_mut().rust_mut().get_mut();
            m.library = library;
            m.diagnostics = diagnostics;
        }
        self.as_mut().rebuild_wires();
        self.as_mut().library_changed();
        self.as_mut().validation_changed();
    }

    fn rebuild_wires(mut self: Pin<&mut Self>) {
        let mut entries: Vec<(String, String, String, i32)> = Vec::new();
        {
            let r = self.as_ref();
            let Some(s) = r.rust().schematic.as_ref() else {
                return;
            };
            let lib = &r.rust().library;
            for inst in &s.instances {
                let module = lib.iter().find(|m| m.name == inst.module_ref);
                for (port, net) in &inst.port_map {
                    if let Some(driver) = net {
                        // Width of the wire = width of the slice if any, else
                        // width of the target port resolved via the instance's
                        // generic-map overrides.
                        let w = match driver {
                            NetRef::InstancePortSlice(_, _, slice)
                            | NetRef::TopPortSlice(_, slice) => match slice {
                                SliceExpr::Bit(_) => 1,
                                SliceExpr::Range { high, low } => (high - low).abs() + 1,
                            },
                            _ => match module {
                                Some(m) => match m.ports.iter().find(|p| &p.name == port) {
                                    Some(p) => resolve_port_width(
                                        &p.port_type,
                                        &m.generics,
                                        &inst.generic_map,
                                    ),
                                    None => -1,
                                },
                                None => -1,
                            },
                        };
                        entries.push((inst.name.clone(), port.clone(), driver.to_key(), w));
                    }
                }
            }
        }
        entries.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let wires: Vec<(String, String, i32)> = entries
            .into_iter()
            .filter_map(|(inst, port, driver, width)| {
                let target = format!("{inst}.{port}");
                // Self-reference: used only as a net identity marker for
                // multi-load undriven signals. Don't render as a wire.
                if driver == target {
                    return None;
                }
                Some((driver, target, width))
            })
            .collect();
        self.as_mut().rust_mut().get_mut().wires = wires;
    }

    fn save_to(mut self: Pin<&mut Self>, path: &Path) -> bool {
        let schematic = match self.as_ref().rust().schematic.clone() {
            Some(s) => s,
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match project::save_project(&schematic, path) {
            Ok(()) => {
                let m = self.as_mut().rust_mut().get_mut();
                m.dirty = false;
                m.last_error.clear();
                self.as_mut().refresh_project_display();
                self.as_mut().project_saved();
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    // --- File ops ---
    pub fn open_project(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        match project::load_project(&path_buf) {
            Ok((schematic, _warnings)) => {
                {
                    let m = self.as_mut().rust_mut().get_mut();
                    m.schematic = Some(schematic);
                    m.current_path = Some(path_buf);
                    m.dirty = false;
                    m.last_error.clear();
                }
                self.as_mut().clear_undo_stacks();
                self.as_mut().refresh_project_display();
                self.as_mut().rebuild_library_and_validate();
                self.as_mut().project_loaded();
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn save_project(mut self: Pin<&mut Self>) -> bool {
        let path = match self.as_ref().rust().current_path.clone() {
            Some(p) => p,
            None => {
                self.as_mut().record_error("no project path set");
                return false;
            }
        };
        self.save_to(&path)
    }

    pub fn save_project_as(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        if self.as_mut().save_to(&path_buf) {
            self.as_mut().rust_mut().get_mut().current_path = Some(path_buf);
            true
        } else {
            false
        }
    }

    pub fn generate_code(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let schematic = match self.as_ref().rust().schematic.clone() {
            Some(s) => s,
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        let library = self.as_ref().rust().library.clone();
        let diagnostics = schematic.validate(&library);

        let result = match schematic.language {
            Language::Vhdl => codegen::vhdl::generate_vhdl(&schematic, &library, &diagnostics),
            Language::SystemVerilog => codegen::sv::generate_sv(&schematic, &library, &diagnostics),
        };

        let code = match result {
            Ok(c) => c,
            Err(codegen::CodegenError::ValidationErrors(errs)) => {
                let mut msg = String::from("validation errors prevent codegen:");
                for d in &errs {
                    msg.push_str("\n  ");
                    msg.push_str(&d.to_string());
                }
                self.as_mut().record_error(msg);
                return false;
            }
            Err(codegen::CodegenError::DirtyInstances(names)) => {
                self.as_mut().record_error(format!(
                    "dirty instances present (source re-parse dropped connections). Reconnect: {}",
                    names.join(", ")
                ));
                return false;
            }
        };

        let path_buf = PathBuf::from(path.to_string());
        match std::fs::write(&path_buf, &code) {
            Ok(()) => {
                self.as_mut().rust_mut().get_mut().last_error.clear();
                true
            }
            Err(e) => {
                self.as_mut().record_error(format!("write {}: {e}", path_buf.display()));
                false
            }
        }
    }

    pub fn suggest_codegen_path(&self) -> QString {
        let schematic = match self.rust().schematic.as_ref() {
            Some(s) => s,
            None => return QString::from(""),
        };
        let ext = match schematic.language {
            Language::Vhdl => "vhd",
            Language::SystemVerilog => "sv",
        };
        let dir = self
            .rust()
            .current_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let filename = format!("{}.{ext}", schematic.top_name);
        let full = dir.join(filename);
        QString::from(full.to_string_lossy().as_ref())
    }

    pub fn project_language(&self) -> i32 {
        match self.rust().schematic.as_ref().map(|s| &s.language) {
            Some(Language::Vhdl) => 0,
            Some(Language::SystemVerilog) => 1,
            None => -1,
        }
    }

    pub fn top_level_buffer(&self) -> QString {
        let Some(schematic) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        // Wrap in language-shaped scaffolding for readability. The parser
        // ignores these decoration tokens, so users can edit them freely.
        // Always trailing comma after every entry — the editor's grammar
        // doesn't try to mimic the language's terminator rules. The parser
        // strips trailing commas anyway. Body grammar (`name : in/out type`,
        // `name : type [:= value]`) is shared across languages.
        let sv = matches!(schematic.language, Language::SystemVerilog);
        let mut out = String::new();
        if sv {
            out.push_str(&format!("module {} (\n", schematic.top_name));
        } else {
            out.push_str(&format!("entity {} is\n", schematic.top_name));
        }
        if !schematic.top_generics.is_empty() {
            if !sv {
                out.push_str("  generic (\n");
            }
            for g in &schematic.top_generics {
                let value = match &g.default_value {
                    Some(v) => format!(" := {v}"),
                    None => String::new(),
                };
                out.push_str(&format!("    {} : {}{},\n", g.name, g.type_name, value));
            }
            if !sv {
                out.push_str("  );\n");
            }
        }
        if !schematic.top_ports.is_empty() {
            if !sv {
                out.push_str("  port (\n");
            }
            for p in &schematic.top_ports {
                let dir = match p.direction {
                    Direction::In => "in",
                    Direction::Out => "out",
                    Direction::InOut => "inout",
                };
                let type_str = port_type_brief(&p.port_type);
                out.push_str(&format!("    {} : {} {},\n", p.name, dir, type_str));
            }
            if !sv {
                out.push_str("  );\n");
            }
        }
        if sv {
            out.push_str(");\nendmodule\n");
        } else {
            out.push_str(&format!("end entity {};\n", schematic.top_name));
        }
        QString::from(out.as_str())
    }

    pub fn commit_top_level_buffer(mut self: Pin<&mut Self>, buffer: &QString) -> bool {
        if self.as_ref().rust().schematic.is_none() {
            self.as_mut().record_error("no project loaded");
            return false;
        }
        let parsed = match parse_top_level_buffer(&buffer.to_string()) {
            Ok(p) => p,
            Err(e) => {
                self.as_mut().record_error(e);
                return false;
            }
        };
        self.as_mut().push_snapshot();
        {
            let m = self.as_mut().rust_mut().get_mut();
            let s = m.schematic.as_mut().expect("checked above");
            let top_name = s.top_name.clone();
            s.replace_top_level(top_name, parsed.generics, parsed.ports);
            m.dirty = true;
            m.last_error.clear();
        }
        self.as_mut().refresh_project_display();
        self.as_mut().rebuild_library_and_validate();
        // Fire project_loaded so the canvas does a full rebuild of top-port
        // graphics (added/removed pins) — onPortMapChanged only relayouts
        // existing items, it doesn't add or drop any.
        self.as_mut().project_loaded();
        true
    }

    pub fn new_project(mut self: Pin<&mut Self>, name: &QString, language: i32) -> bool {
        let lang = match language {
            0 => Language::Vhdl,
            1 => Language::SystemVerilog,
            _ => return false,
        };
        {
            let m = self.as_mut().rust_mut().get_mut();
            m.schematic = Some(Schematic::new(name.to_string(), lang));
            m.current_path = None;
            m.dirty = true;
            m.last_error.clear();
            m.library.clear();
            m.diagnostics.clear();
        }
        self.as_mut().clear_undo_stacks();
        self.as_mut().refresh_project_display();
        self.as_mut().project_loaded();
        self.as_mut().validation_changed();
        true
    }

    pub fn has_project(&self) -> bool {
        self.rust().schematic.is_some()
    }

    pub fn last_error(&self) -> QString {
        QString::from(&self.rust().last_error)
    }

    // --- Instance inspection ---
    pub fn instance_count(&self) -> i32 {
        self.rust()
            .schematic
            .as_ref()
            .map(|s| s.instances.len() as i32)
            .unwrap_or(0)
    }

    pub fn instance_name(&self, index: i32) -> QString {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.get(index as usize))
            .map(|i| QString::from(&i.name))
            .unwrap_or_default()
    }

    pub fn instance_module(&self, index: i32) -> QString {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.get(index as usize))
            .map(|i| QString::from(&i.module_ref))
            .unwrap_or_default()
    }

    // --- Instance mutation ---
    pub fn add_instance(mut self: Pin<&mut Self>, name: &QString, module: &QString) -> bool {
        let name_s = name.to_string();
        let module_s = module.to_string();
        // Batch-aware so drop (add + initial position) is one undo step.
        // Signals still fire immediately — the canvas must create the item
        // before the position arrives.
        self.as_mut().snapshot_for_mutation();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.add_instance(name_s.clone(), module_s).map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                self.as_mut().rust_mut().get_mut().dirty = true;
                self.as_mut().set_dirty(true);
                self.as_mut().instance_added(QString::from(&name_s));
                self.as_mut().rebuild_library_and_validate();
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn rename_instance(
        mut self: Pin<&mut Self>,
        old_name: &QString,
        new_name: &QString,
    ) -> bool {
        let old_s = old_name.to_string();
        let new_s = new_name.to_string();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.rename_instance(&old_s, &new_s),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                {
                    let r = self.as_mut().rust_mut().get_mut();
                    r.dirty = true;
                    if r.selected_instance == old_s {
                        r.selected_instance = new_s.clone();
                    }
                }
                self.as_mut().set_dirty(true);
                // Rebuild the wire cache BEFORE the canvas-facing signals fire
                // — onInstanceRemoved/Added trigger rebuildWires which reads
                // from the cache. Stale cache = wires referencing the old
                // name vanish.
                self.as_mut().rebuild_library_and_validate();
                self.as_mut().instance_removed(QString::from(&old_s));
                self.as_mut().instance_added(QString::from(&new_s));
                self.as_mut().selection_changed(QString::from(&new_s));
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn instance_is_dirty(&self, index: i32) -> bool {
        let Some(s) = self.rust().schematic.as_ref() else { return false };
        s.instances.get(index as usize).map(|i| i.dirty).unwrap_or(false)
    }

    pub fn instance_index(&self, name: &QString) -> i32 {
        let n = name.to_string();
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.iter().position(|i| i.name == n))
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    pub fn instance_is_dirty_name(&self, name: &QString) -> bool {
        let n = name.to_string();
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.iter().find(|i| i.name == n))
            .map(|i| i.dirty)
            .unwrap_or(false)
    }

    pub fn set_instance_position(
        mut self: Pin<&mut Self>,
        name: &QString,
        x: f64,
        y: f64,
    ) -> bool {
        let name_s = name.to_string();
        let exists = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .is_some_and(|s| s.instances.iter().any(|i| i.name == name_s));
        if !exists {
            return false;
        }
        // Snapshot so a completed move is undoable. Called once per drag
        // release / drop — never per drag tick.
        self.as_mut().snapshot_for_mutation();
        if let Some(inst) = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .schematic
            .as_mut()
            .and_then(|s| s.get_instance_mut(&name_s))
        {
            inst.position = (x as f32, y as f32);
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut()
            .instance_moved(QString::from(&name_s), x, y);
        true
    }

    pub fn instance_pos_x(&self, index: i32) -> f64 {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.get(index as usize))
            .map(|i| i.position.0 as f64)
            .unwrap_or(0.0)
    }

    pub fn instance_pos_y(&self, index: i32) -> f64 {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.get(index as usize))
            .map(|i| i.position.1 as f64)
            .unwrap_or(0.0)
    }

    pub fn set_selected_instance(mut self: Pin<&mut Self>, name: &QString) {
        let name_s = name.to_string();
        self.as_mut().rust_mut().get_mut().selected_instance = name_s.clone();
        self.as_mut().selection_changed(QString::from(&name_s));
    }

    pub fn selected_instance(&self) -> QString {
        QString::from(&self.rust().selected_instance)
    }

    pub fn instance_source_path(&self, index: i32) -> QString {
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else {
            return QString::default();
        };
        let Some(inst) = s.instances.get(index as usize) else {
            return QString::default();
        };
        let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
            return QString::default();
        };
        QString::from(&module.source_path.to_string_lossy().into_owned())
    }

    pub fn remove_instance(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        self.as_mut().push_snapshot();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.remove_instance(&name_s).map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                self.as_mut().rust_mut().get_mut().dirty = true;
                self.as_mut().set_dirty(true);
                // Refresh wire cache BEFORE firing signals so canvas reads fresh data.
                self.as_mut().rebuild_library_and_validate();
                self.as_mut().instance_removed(QString::from(&name_s));
                self.as_mut().port_map_changed_bulk();
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn set_port_map_entry(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
        rhs: &QString,
    ) -> bool {
        let inst = instance.to_string();
        let port_s = port.to_string();
        let parsed = parse_net_rhs(&rhs.to_string());
        self.as_mut().snapshot_for_mutation();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.set_port_map_entry(&inst, port_s.clone(), parsed).map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                self.as_mut().finish_mutation(Some((&inst, &port_s)));
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn set_port_map_entry_slice(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
        driver_inst: &QString,
        driver_port: &QString,
        slice_high: i32,
        slice_low: i32,
    ) -> bool {
        let inst = instance.to_string();
        let port_s = port.to_string();
        let di = driver_inst.to_string();
        let dp = driver_port.to_string();
        let slice = if slice_high == slice_low {
            SliceExpr::Bit(slice_high)
        } else {
            SliceExpr::Range {
                high: slice_high,
                low: slice_low,
            }
        };
        // Empty driver_inst means TopPortSlice; otherwise InstancePortSlice.
        let net_ref = if di.is_empty() {
            NetRef::TopPortSlice(dp.clone(), slice)
        } else {
            NetRef::InstancePortSlice(di, dp.clone(), slice)
        };
        self.as_mut().snapshot_for_mutation();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s
                .set_port_map_entry(&inst, port_s.clone(), Some(net_ref))
                .map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                self.as_mut().finish_mutation(Some((&inst, &port_s)));
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn set_consumer_slice(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
        slice_high: i32,
        slice_low: i32,
    ) -> bool {
        let inst = instance.to_string();
        let port_s = port.to_string();
        let slice = if slice_high == slice_low {
            SliceExpr::Bit(slice_high)
        } else {
            SliceExpr::Range {
                high: slice_high,
                low: slice_low,
            }
        };
        self.as_mut().snapshot_for_mutation();
        let updated = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => match s.instances.iter_mut().find(|i| i.name == inst) {
                Some(i) => {
                    i.consumer_slices.insert(port_s.clone(), slice);
                    true
                }
                None => false,
            },
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        if !updated {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().finish_mutation(Some((&inst, &port_s)));
        true
    }

    pub fn clear_consumer_slice(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
    ) -> bool {
        let inst = instance.to_string();
        let port_s = port.to_string();
        self.as_mut().snapshot_for_mutation();
        let removed = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => match s.instances.iter_mut().find(|i| i.name == inst) {
                Some(i) => i.consumer_slices.remove(&port_s).is_some(),
                None => false,
            },
            None => return false,
        };
        if removed {
            self.as_mut().finish_mutation(Some((&inst, &port_s)));
        }
        true
    }

    pub fn consumer_slice(&self, instance: &QString, port: &QString) -> QString {
        let inst = instance.to_string();
        let port_s = port.to_string();
        let Some(s) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        let Some(i) = s.instances.iter().find(|i| i.name == inst) else {
            return QString::default();
        };
        match i.consumer_slices.get(&port_s) {
            Some(slice) => QString::from(&slice.to_suffix()),
            None => QString::default(),
        }
    }

    pub fn promote_port_to_top(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
    ) -> QString {
        let inst_s = instance.to_string();
        let port_s = port.to_string();
        self.as_mut().push_snapshot();

        // Phase 1: gather the source port def and compute the resolved top-port
        // name. Collect any error as a string first so we release the immutable
        // borrow on `self` before calling `record_error`.
        let lookup: Result<(PortDef, String, bool), String> = {
            let this = self.as_ref();
            let r = this.rust();
            match r.schematic.as_ref() {
                None => Err("no project loaded".to_string()),
                Some(schematic) => {
                    match schematic.instances.iter().find(|i| i.name == inst_s) {
                        None => Err(format!("instance not found: {inst_s}")),
                        Some(inst) => match r.library.iter().find(|m| m.name == inst.module_ref) {
                            None => Err(format!("module not in library: {}", inst.module_ref)),
                            Some(module) => match module.ports.iter().find(|p| p.name == port_s) {
                                None => Err(format!("port not found: {inst_s}.{port_s}")),
                                Some(p) => {
                                    let pd = p.clone();
                                    let (name, create) =
                                        resolve_top_port_name(&schematic.top_ports, &pd);
                                    Ok((pd, name, create))
                                }
                            },
                        },
                    }
                }
            }
        };
        let (port_def, resolved_name, need_create) = match lookup {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().record_error(e);
                return QString::default();
            }
        };

        // Phase 2: mutate.
        {
            let s = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
                .expect("checked in phase 1");
            if need_create {
                let mut new_port = port_def.clone();
                new_port.name = resolved_name.clone();
                s.top_ports.push(new_port);
            }
            // Rewrite every port_map entry that sat on the same net as the
            // promoted pin — the net identity changes from InstancePort(inst, port)
            // to TopPort(resolved_name). Preserves slice suffixes where present.
            let old_net = NetRef::InstancePort(inst_s.clone(), port_s.clone());
            let new_top = NetRef::TopPort(resolved_name.clone());
            for other in s.instances.iter_mut() {
                for entry in other.port_map.values_mut() {
                    if let Some(net) = entry
                        && net.base() == old_net
                    {
                        let rewritten = match net {
                            NetRef::InstancePort(_, _) => new_top.clone(),
                            NetRef::InstancePortSlice(_, _, slice) => {
                                NetRef::TopPortSlice(resolved_name.clone(), slice.clone())
                            }
                            other => other.clone(),
                        };
                        *entry = Some(rewritten);
                    }
                }
            }
            // Finally set the promoted pin itself to the new top-port.
            if let Err(e) = s.set_port_map_entry(
                &inst_s,
                port_s.clone(),
                Some(new_top),
            ) {
                self.as_mut().record_error(e.to_string());
                return QString::default();
            }
        }

        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        // Rebuild the wire cache BEFORE firing project_loaded — the canvas's
        // rebuild handler reads from that cache to render wires, and a stale
        // cache means the freshly-promoted wire wouldn't show.
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().project_loaded();
        QString::from(&resolved_name)
    }

    pub fn create_manual_bundle(
        mut self: Pin<&mut Self>,
        instance: &QString,
        name: &QString,
        ports_csv: &QString,
    ) -> bool {
        let inst_s = instance.to_string();
        let name_s = name.to_string().trim().to_string();
        if name_s.is_empty() {
            self.as_mut().record_error("bundle name is empty");
            return false;
        }
        let ports: Vec<String> = ports_csv
            .to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if ports.is_empty() {
            self.as_mut().record_error("no ports specified for bundle");
            return false;
        }
        self.as_mut().push_snapshot();
        {
            let s = match self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
            {
                Some(s) => s,
                None => {
                    self.as_mut().record_error("no project loaded");
                    return false;
                }
            };
            let Some(inst) = s.instances.iter_mut().find(|i| i.name == inst_s) else {
                self.as_mut()
                    .record_error(format!("instance not found: {inst_s}"));
                return false;
            };
            inst.manual_bundles.insert(name_s, ports);
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().port_map_changed_bulk();
        true
    }

    pub fn remove_manual_bundle(
        mut self: Pin<&mut Self>,
        instance: &QString,
        name: &QString,
    ) -> bool {
        let inst_s = instance.to_string();
        let name_s = name.to_string();
        self.as_mut().push_snapshot();
        let removed = {
            let s = match self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
            {
                Some(s) => s,
                None => {
                    self.as_mut().record_error("no project loaded");
                    return false;
                }
            };
            let Some(inst) = s.instances.iter_mut().find(|i| i.name == inst_s) else {
                self.as_mut()
                    .record_error(format!("instance not found: {inst_s}"));
                return false;
            };
            inst.manual_bundles.remove(&name_s).is_some()
        };
        if removed {
            self.as_mut().rust_mut().get_mut().dirty = true;
            self.as_mut().set_dirty(true);
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().port_map_changed_bulk();
        }
        removed
    }

    pub fn manual_bundle_count(&self, instance_index: i32) -> i32 {
        let Some(s) = self.rust().schematic.as_ref() else { return 0 };
        s.instances
            .get(instance_index as usize)
            .map(|i| i.manual_bundles.len() as i32)
            .unwrap_or(0)
    }

    fn manual_bundle_at(
        &self,
        instance_index: i32,
        bundle_index: i32,
    ) -> Option<(&String, &Vec<String>)> {
        let s = self.rust().schematic.as_ref()?;
        let inst = s.instances.get(instance_index as usize)?;
        // Deterministic order: sort keys so repeated invokable calls align.
        let mut keys: Vec<&String> = inst.manual_bundles.keys().collect();
        keys.sort();
        let key = keys.get(bundle_index as usize)?;
        let ports = inst.manual_bundles.get(*key)?;
        Some((key, ports))
    }

    pub fn manual_bundle_name(
        &self,
        instance_index: i32,
        bundle_index: i32,
    ) -> QString {
        self.manual_bundle_at(instance_index, bundle_index)
            .map(|(k, _)| QString::from(k))
            .unwrap_or_default()
    }

    pub fn manual_bundle_port_count(
        &self,
        instance_index: i32,
        bundle_index: i32,
    ) -> i32 {
        self.manual_bundle_at(instance_index, bundle_index)
            .map(|(_, ports)| ports.len() as i32)
            .unwrap_or(0)
    }

    pub fn manual_bundle_port_name(
        &self,
        instance_index: i32,
        bundle_index: i32,
        port_index: i32,
    ) -> QString {
        self.manual_bundle_at(instance_index, bundle_index)
            .and_then(|(_, ports)| ports.get(port_index as usize))
            .map(QString::from)
            .unwrap_or_default()
    }

    /// For each currently-unconnected port on `instance`, connect it to a
    /// top-level port with the same name, direction, and width-compatible type.
    /// Returns the number of connections made. Never runs automatically; the
    /// user invokes this explicitly (toolbar / Ctrl+M) — matches the "don't
    /// guess at wiring on drop" rule.
    pub fn match_by_name(mut self: Pin<&mut Self>, instance: &QString) -> i32 {
        let inst_s = instance.to_string();
        self.as_mut().push_snapshot();

        // Phase 1: gather the set of (port_name, top_port_name) pairs to wire,
        // reading from the current schematic + library. All reads.
        let pairs: Vec<(String, String)> = {
            let this = self.as_ref();
            let r = this.rust();
            let Some(s) = r.schematic.as_ref() else {
                return 0;
            };
            let Some(inst) = s.instances.iter().find(|i| i.name == inst_s) else {
                return 0;
            };
            let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
                return 0;
            };
            module
                .ports
                .iter()
                .filter_map(|p| {
                    // Only consider unconnected ports.
                    if let Some(Some(_)) = inst.port_map.get(&p.name) { return None }
                    // Look for a top-port with same name and compatible dir/type.
                    let tp = s.top_ports.iter().find(|tp| {
                        tp.name == p.name
                            && tp.direction == p.direction
                            && tp.port_type == p.port_type
                    })?;
                    Some((p.name.clone(), tp.name.clone()))
                })
                .collect()
        };

        if pairs.is_empty() {
            return 0;
        }

        // Phase 2: apply all at once.
        let mut count: i32 = 0;
        {
            let s = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
                .expect("checked in phase 1");
            for (port_name, top_name) in &pairs {
                if s.set_port_map_entry(
                    &inst_s,
                    port_name.clone(),
                    Some(NetRef::TopPort(top_name.clone())),
                )
                .is_ok()
                {
                    count += 1;
                }
            }
        }

        if count > 0 {
            self.as_mut().rust_mut().get_mut().dirty = true;
            self.as_mut().set_dirty(true);
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().port_map_changed_bulk();
        }
        count
    }

    pub fn reload_library(mut self: Pin<&mut Self>) -> bool {
        if self.as_ref().rust().schematic.is_none() {
            return false;
        }
        self.as_mut().push_snapshot();
        // Explicit refresh must bypass the mtime check (e.g. sub-second
        // saves, or tools that preserve mtime), so drop the parse cache.
        self.as_mut().rust_mut().get_mut().parse_cache.clear();

        // Snapshot the current library + re-parse to get the new library.
        // Compare: for each instance whose module's ports changed, drop the
        // stale port_map entries and mark the instance dirty.
        let old_library: Vec<ModuleDef> = self.as_ref().rust().library.clone();
        let new_library: Vec<ModuleDef> = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .map(|s| {
                let (lib, _errors) = s.resolve_modules();
                lib
            })
            .unwrap_or_default();

        let newly_dirty = {
            let s = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
                .expect("checked above");
            s.apply_library_update(&old_library, &new_library)
        };

        self.as_mut().rebuild_library_and_validate();
        // Full canvas rebuild so updated widths / generics / port lists reflect.
        self.as_mut().project_loaded();

        if !newly_dirty.is_empty() {
            self.as_mut().record_error(format!(
                "library re-parse dropped connections on: {}",
                newly_dirty.join(", ")
            ));
        }
        true
    }

    /// Clear the dirty flag on an instance. User signals they've reviewed
    /// the dropped-port fallout from a library re-parse.
    pub fn clear_instance_dirty(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        let was_dirty = {
            let Some(s) = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
            else {
                return false;
            };
            s.clear_instance_dirty(&name_s)
        };
        if was_dirty {
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().project_loaded();
        }
        was_dirty
    }

    /// Snapshot the current schematic for undo. Called as the first line
    /// of every model-mutating invokable. No-op when no project is loaded.
    /// Wipes the redo stack — a new edit invalidates redo history.
    fn push_snapshot(mut self: Pin<&mut Self>) {
        let snap = match self.as_ref().rust().schematic.as_ref() {
            Some(s) => match serde_json::to_string(s) {
                Ok(json) => json,
                Err(_) => return,
            },
            None => return,
        };
        let m = self.as_mut().rust_mut().get_mut();
        m.undo_stack.push(snap);
        if m.undo_stack.len() > UNDO_STACK_LIMIT {
            m.undo_stack.remove(0);
        }
        m.redo_stack.clear();
    }

    fn clear_undo_stacks(mut self: Pin<&mut Self>) {
        let m = self.as_mut().rust_mut().get_mut();
        m.undo_stack.clear();
        m.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.rust().undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.rust().redo_stack.is_empty()
    }

    pub fn undo(mut self: Pin<&mut Self>) -> bool {
        // Move current state onto redo stack, restore prior state.
        let prior = match self.as_mut().rust_mut().get_mut().undo_stack.pop() {
            Some(s) => s,
            None => return false,
        };
        let restored: Schematic = match serde_json::from_str(&prior) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let current = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        {
            let m = self.as_mut().rust_mut().get_mut();
            if let Some(c) = current {
                m.redo_stack.push(c);
                if m.redo_stack.len() > UNDO_STACK_LIMIT {
                    m.redo_stack.remove(0);
                }
            }
            m.schematic = Some(restored);
            m.dirty = true;
        }
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().project_loaded();
        true
    }

    pub fn redo(mut self: Pin<&mut Self>) -> bool {
        let next = match self.as_mut().rust_mut().get_mut().redo_stack.pop() {
            Some(s) => s,
            None => return false,
        };
        let restored: Schematic = match serde_json::from_str(&next) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let current = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        {
            let m = self.as_mut().rust_mut().get_mut();
            if let Some(c) = current {
                m.undo_stack.push(c);
                if m.undo_stack.len() > UNDO_STACK_LIMIT {
                    m.undo_stack.remove(0);
                }
            }
            m.schematic = Some(restored);
            m.dirty = true;
        }
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().project_loaded();
        true
    }

    pub fn module_generic_count(&self, instance_index: i32) -> i32 {
        self.instance_module_def(instance_index)
            .map(|m| m.generics.len() as i32)
            .unwrap_or(0)
    }

    pub fn module_generic_name(
        &self,
        instance_index: i32,
        generic_index: i32,
    ) -> QString {
        self.instance_module_def(instance_index)
            .and_then(|m| m.generics.get(generic_index as usize))
            .map(|g| QString::from(&g.name))
            .unwrap_or_default()
    }

    pub fn module_generic_default(
        &self,
        instance_index: i32,
        generic_index: i32,
    ) -> QString {
        self.instance_module_def(instance_index)
            .and_then(|m| m.generics.get(generic_index as usize))
            .and_then(|g| g.default_value.as_ref())
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn generic_map_entry(&self, instance: &QString, generic_name: &QString) -> QString {
        let inst_s = instance.to_string();
        let gen_s = generic_name.to_string();
        let Some(s) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        let Some(inst) = s.instances.iter().find(|i| i.name == inst_s) else {
            return QString::default();
        };
        inst.generic_map
            .get(&gen_s)
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn port_map_entry(&self, instance: &QString, port: &QString) -> QString {
        let inst_s = instance.to_string();
        let port_s = port.to_string();
        let Some(s) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        let Some(inst) = s.instances.iter().find(|i| i.name == inst_s) else {
            return QString::default();
        };
        match inst.port_map.get(&port_s) {
            Some(Some(net)) => {
                // Return in the same form `parse_net_rhs` accepts.
                match net {
                    NetRef::TopPort(n) => QString::from(n),
                    NetRef::InstancePort(i, p) => QString::from(&format!("{i}.{p}")),
                    // Slice variants: encode with bracket suffix. parse_net_rhs
                    // doesn't currently parse these back — future slice-connect
                    // work will round-trip via a different invokable.
                    NetRef::InstancePortSlice(_, _, _)
                    | NetRef::TopPortSlice(_, _) => QString::from(&net.to_key()),
                }
            }
            _ => QString::default(),
        }
    }

    pub fn clear_port_map_entry(
        mut self: Pin<&mut Self>,
        instance: &QString,
        port: &QString,
    ) -> bool {
        let inst = instance.to_string();
        let port_s = port.to_string();
        self.as_mut().snapshot_for_mutation();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.set_port_map_entry(&inst, port_s.clone(), None).map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                self.as_mut().finish_mutation(Some((&inst, &port_s)));
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    pub fn set_generic_map_entry(
        mut self: Pin<&mut Self>,
        instance: &QString,
        generic: &QString,
        expr: &QString,
    ) -> bool {
        let inst = instance.to_string();
        let gen_s = generic.to_string();
        let expr_s = expr.to_string();
        self.as_mut().snapshot_for_mutation();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.set_generic_map_entry(&inst, gen_s, expr_s).map(|_| ()),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        match result {
            Ok(()) => {
                // Generic override can change port widths — canvas needs to
                // re-layout pins + width badges, so emit the bulk signal.
                self.as_mut().finish_mutation(None);
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    // --- Port metadata ---
    pub fn instance_port_count(&self, instance_index: i32) -> i32 {
        self.instance_module_ports(instance_index)
            .map(|p| p.len() as i32)
            .unwrap_or(0)
    }

    pub fn instance_port_name(&self, instance_index: i32, port_index: i32) -> QString {
        self.instance_port_at(instance_index, port_index)
            .map(|p| QString::from(&p.name))
            .unwrap_or_default()
    }

    pub fn instance_port_direction(&self, instance_index: i32, port_index: i32) -> i32 {
        self.instance_port_at(instance_index, port_index)
            .map(|p| direction_code(&p.direction))
            .unwrap_or(-1)
    }

    pub fn instance_port_width(&self, instance_index: i32, port_index: i32) -> i32 {
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else { return 0 };
        let Some(inst) = s.instances.get(instance_index as usize) else { return 0 };
        let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
            return 0;
        };
        let Some(port) = module.ports.get(port_index as usize) else { return 0 };
        resolve_port_width(&port.port_type, &module.generics, &inst.generic_map)
    }

    pub fn instance_port_bundle(&self, instance_index: i32, port_index: i32) -> QString {
        self.instance_port_at(instance_index, port_index)
            .and_then(|p| p.bundle.as_ref())
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn instance_dependency_count(&self, instance_index: i32) -> i32 {
        self.instance_module_def(instance_index)
            .map(|m| m.dependencies.len() as i32)
            .unwrap_or(0)
    }

    pub fn instance_dependency_name(&self, instance_index: i32, dep_index: i32) -> QString {
        self.instance_module_def(instance_index)
            .and_then(|m| m.dependencies.get(dep_index as usize))
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn instance_dependency_present(&self, instance_index: i32, dep_index: i32) -> bool {
        let Some(m) = self.instance_module_def(instance_index) else {
            return false;
        };
        let Some(dep_name) = m.dependencies.get(dep_index as usize) else {
            return false;
        };
        self.rust().library.iter().any(|lm| lm.name == *dep_name)
    }

    pub fn top_port_count(&self) -> i32 {
        self.rust()
            .schematic
            .as_ref()
            .map(|s| s.top_ports.len() as i32)
            .unwrap_or(0)
    }

    pub fn top_port_name(&self, index: i32) -> QString {
        self.top_port_at(index)
            .map(|p| QString::from(&p.name))
            .unwrap_or_default()
    }

    pub fn top_port_direction(&self, index: i32) -> i32 {
        self.top_port_at(index)
            .map(|p| direction_code(&p.direction))
            .unwrap_or(-1)
    }

    pub fn top_port_width(&self, index: i32) -> i32 {
        self.top_port_at(index)
            .map(|p| port_type_width(&p.port_type))
            .unwrap_or(0)
    }

    // --- Wires ---
    pub fn wire_count(&self) -> i32 {
        self.rust().wires.len() as i32
    }

    pub fn wire_source(&self, index: i32) -> QString {
        self.rust()
            .wires
            .get(index as usize)
            .map(|(src, _, _)| QString::from(src))
            .unwrap_or_default()
    }

    pub fn wire_target(&self, index: i32) -> QString {
        self.rust()
            .wires
            .get(index as usize)
            .map(|(_, tgt, _)| QString::from(tgt))
            .unwrap_or_default()
    }

    pub fn wire_width(&self, index: i32) -> i32 {
        self.rust()
            .wires
            .get(index as usize)
            .map(|(_, _, w)| *w)
            .unwrap_or(0)
    }

    pub fn set_alias(
        mut self: Pin<&mut Self>,
        net_key: &QString,
        alias: &QString,
    ) -> bool {
        let key_s = net_key.to_string();
        let alias_s = alias.to_string();
        let Some(net_ref) = NetRef::from_key(&key_s) else {
            self.as_mut().record_error("invalid net key");
            return false;
        };
        self.as_mut().push_snapshot();
        let ok = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => {
                if alias_s.trim().is_empty() {
                    s.remove_alias(&net_ref);
                } else {
                    s.set_alias(net_ref, alias_s);
                }
                true
            }
            None => false,
        };
        if !ok {
            self.as_mut().record_error("no project loaded");
            return false;
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().alias_changed(QString::from(&key_s));
        true
    }

    // --- Library ---
    pub fn library_module_count(&self) -> i32 {
        self.rust().library.len() as i32
    }

    pub fn library_module_name(&self, index: i32) -> QString {
        self.rust()
            .library
            .get(index as usize)
            .map(|m| QString::from(&m.name))
            .unwrap_or_default()
    }

    pub fn add_library_path(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        let added = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.add_library_path(path_buf),
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        };
        if !added {
            return false;
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        true
    }

    pub fn library_path_count(&self) -> i32 {
        self.rust()
            .schematic
            .as_ref()
            .map(|s| s.library_paths.len() as i32)
            .unwrap_or(0)
    }

    pub fn library_path(&self, index: i32) -> QString {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.library_paths.get(index as usize))
            .map(|p| QString::from(&p.to_string_lossy().into_owned()))
            .unwrap_or_default()
    }

    pub fn current_project_path(&self) -> QString {
        self.rust()
            .current_path
            .as_ref()
            .map(|p| QString::from(&p.to_string_lossy().into_owned()))
            .unwrap_or_default()
    }

    pub fn remove_library_path(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        let removed = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s.remove_library_path(&path_buf),
            None => return false,
        };
        if !removed {
            return false;
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        true
    }

    // --- Validation ---
    pub fn validation_count(&self) -> i32 {
        self.rust().diagnostics.len() as i32
    }

    pub fn validation_error_count(&self) -> i32 {
        self.rust()
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .count() as i32
    }

    pub fn validation_warning_count(&self) -> i32 {
        self.rust()
            .diagnostics
            .iter()
            .filter(|d| !d.is_error())
            .count() as i32
    }

    pub fn validation_message(&self, index: i32) -> QString {
        self.rust()
            .diagnostics
            .get(index as usize)
            .map(|d| QString::from(&d.to_string()))
            .unwrap_or_default()
    }
}

fn direction_code(d: &Direction) -> i32 {
    match d {
        Direction::In => 0,
        Direction::Out => 1,
        Direction::InOut => 2,
    }
}

/// Pure helper for top-port name resolution during promotion.
/// Returns (resolved_name, should_create_new_top_port).
/// If an existing top-port with matching name AND direction+type+bundle exists,
/// it is reused; otherwise a numeric suffix (`_1`, `_2`, …) is appended until
/// the name is unique.
pub(super) fn resolve_top_port_name(
    existing: &[PortDef],
    src: &PortDef,
) -> (String, bool) {
    let matches = |tp: &PortDef| {
        tp.direction == src.direction
            && tp.port_type == src.port_type
            && tp.bundle == src.bundle
    };
    if let Some(tp) = existing.iter().find(|tp| tp.name == src.name) {
        if matches(tp) {
            return (src.name.clone(), false);
        }
        let mut candidate = format!("{}_1", src.name);
        let mut counter = 2;
        while existing.iter().any(|tp| tp.name == candidate) {
            candidate = format!("{}_{}", src.name, counter);
            counter += 1;
        }
        return (candidate, true);
    }
    (src.name.clone(), true)
}

fn port_type_width(t: &PortType) -> i32 {
    // Returns 0 for scalar (std_logic), N>0 for a resolved vector of N bits,
    // and -1 for a vector whose bounds are not literal (e.g. `WIDTH-1 downto 0`).
    match t {
        PortType::StdLogic => 0,
        PortType::StdLogicVector(Range { high, low, .. }) => match (high, low) {
            (RangeExpr::Literal(h), RangeExpr::Literal(l)) => {
                ((h - l).abs() as i32 + 1).max(1)
            }
            _ => -1,
        },
        _ => 0,
    }
}

/// Resolve an instance port's width using the module's generic defaults and
/// any instance-level generic-map overrides. Returns the same 0 / N / -1
/// contract as `port_type_width`.
fn resolve_port_width(
    port_type: &PortType,
    module_generics: &[crate::types::GenericDef],
    instance_generic_map: &std::collections::HashMap<String, String>,
) -> i32 {
    let resolved = codegen::resolve_port_type(port_type, module_generics, instance_generic_map);
    port_type_width(&resolved)
}

fn parse_net_rhs(rhs: &str) -> Option<NetRef> {
    let trimmed = rhs.trim();
    if trimmed.is_empty() || trimmed == "open" {
        return None;
    }
    // Strip optional bracket suffix `[h:l]` or `[i]` and parse it.
    let (head, slice) = match trimmed.find('[') {
        Some(open) => {
            let close = trimmed.find(']')?;
            if close <= open {
                return None;
            }
            let inner = trimmed[open + 1..close].trim();
            let slice = if let Some((h, l)) = inner.split_once(':') {
                let high: i32 = h.trim().parse().ok()?;
                let low: i32 = l.trim().parse().ok()?;
                if high == low {
                    SliceExpr::Bit(high)
                } else {
                    SliceExpr::Range { high, low }
                }
            } else {
                SliceExpr::Bit(inner.parse().ok()?)
            };
            (trimmed[..open].trim(), Some(slice))
        }
        None => (trimmed, None),
    };
    if let Some((inst, port)) = head.split_once('.') {
        let inst_s = inst.trim().to_string();
        let port_s = port.trim().to_string();
        return Some(match slice {
            Some(s) => NetRef::InstancePortSlice(inst_s, port_s, s),
            None => NetRef::InstancePort(inst_s, port_s),
        });
    }
    Some(match slice {
        Some(s) => NetRef::TopPortSlice(head.to_string(), s),
        None => NetRef::TopPort(head.to_string()),
    })
}

pub(super) struct ParsedTopLevel {
    pub generics: Vec<crate::types::GenericDef>,
    pub ports: Vec<PortDef>,
}

/// Parse the free-form top-level buffer. One declaration per line; ordering
/// determines display order. Decoration tokens (`entity ... is`, `port (`,
/// `);`, `end entity;`, blank lines, `--` comments) are ignored. Each
/// remaining line is `<name> : <rest>`:
///   - if `<rest>` begins with `in` / `out` / `inout`  → port
///   - otherwise                                       → generic
///
/// Port: `name : direction [type]`. Omitted type defaults to `std_logic`.
/// Generic: `name : type [:= value]`.
pub(super) fn parse_top_level_buffer(buffer: &str) -> Result<ParsedTopLevel, String> {
    use crate::types::{GenericDef, PortDef, PortType};
    let mut generics: Vec<GenericDef> = Vec::new();
    let mut ports: Vec<PortDef> = Vec::new();

    for (i, raw_line) in buffer.lines().enumerate() {
        let line_no = i + 1;
        // Strip line comments — VHDL `--` and SV `//`.
        let stripped = match raw_line.split_once("--") {
            Some((before, _)) => before,
            None => match raw_line.split_once("//") {
                Some((before, _)) => before,
                None => raw_line,
            },
        };
        let mut line = stripped.trim().trim_end_matches(',');
        if line.is_empty() {
            continue;
        }
        // Decoration we silently skip — VHDL (`entity / end / generic / port`)
        // and SV (`module / endmodule`) shapes.
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("entity ")
            || lower.starts_with("end ")
            || lower == "end"
            || lower == "endmodule"
            || lower.starts_with("module ")
            || (lower.starts_with("generic") && line.contains('('))
            || (lower.starts_with("port") && line.contains('('))
            || line == ")"
            || line == ");"
        {
            continue;
        }
        if line.ends_with(';') {
            line = &line[..line.len() - 1];
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (lhs, rhs) = match line.split_once(':') {
            Some(p) => p,
            None => {
                return Err(format!(
                    "line {line_no}: expected '<name> : ...' but got '{line}'"
                ));
            }
        };
        let name = lhs.trim().to_string();
        if name.is_empty() {
            return Err(format!("line {line_no}: missing name"));
        }
        let rhs = rhs.trim();
        let (first_tok, rest_tok) = match rhs.split_once(char::is_whitespace) {
            Some((a, b)) => (a.trim(), b.trim()),
            None => (rhs, ""),
        };
        let direction = match first_tok.to_ascii_lowercase().as_str() {
            "in" => Some(Direction::In),
            "out" => Some(Direction::Out),
            "inout" => Some(Direction::InOut),
            _ => None,
        };
        if let Some(direction) = direction {
            let port_type = if rest_tok.is_empty() {
                PortType::StdLogic
            } else {
                parse_port_type(rest_tok)?
            };
            ports.push(PortDef {
                name,
                direction,
                port_type,
                bundle: None,
            });
        } else {
            // Generic. RHS is `type [:= value]`.
            let (type_part, default) = match rhs.split_once(":=") {
                Some((t, v)) => (t.trim().to_string(), Some(v.trim().to_string())),
                None => (rhs.to_string(), None),
            };
            if type_part.is_empty() {
                return Err(format!("line {line_no}: '{name}' missing type"));
            }
            generics.push(GenericDef {
                name,
                type_name: type_part,
                default_value: default,
            });
        }
    }

    Ok(ParsedTopLevel { generics, ports })
}

/// Parse a VHDL-shaped type string. Accepts the brief mini-editor form
/// (`logic`, `logic[7:0]`) AND legacy / pasted VHDL (`std_logic`,
/// `std_logic_vector(7 downto 0)`, `slv(7:0)`). Inside `[...]` or `(...)`,
/// `:` means `downto` (descending — the SV convention); `to` still parses
/// for the rare ascending case.
fn parse_port_type(s: &str) -> Result<PortType, String> {
    let s = s.trim().trim_end_matches(';').trim();
    if s.eq_ignore_ascii_case("std_logic") || s.eq_ignore_ascii_case("logic") {
        return Ok(PortType::StdLogic);
    }
    // SV-style brief: `logic[h:l]`
    let bracket_inside = s
        .strip_prefix("logic")
        .or_else(|| s.strip_prefix("LOGIC"))
        .map(|r| r.trim())
        .and_then(|r| r.strip_prefix('['))
        .and_then(|r| r.strip_suffix(']'));
    if let Some(inside) = bracket_inside {
        let (h, l) = inside
            .split_once(':')
            .ok_or_else(|| format!("type '{s}': expected '<h>:<l>' inside brackets"))?;
        let high = parse_range_bound(h.trim());
        let low = parse_range_bound(l.trim());
        return Ok(PortType::StdLogicVector(Range {
            high,
            low,
            dir: RangeDir::Downto,
        }));
    }
    let inside_opt = s
        .strip_prefix("std_logic_vector")
        .or_else(|| s.strip_prefix("STD_LOGIC_VECTOR"))
        .or_else(|| s.strip_prefix("slv"))
        .or_else(|| s.strip_prefix("SLV"))
        .map(|r| r.trim())
        .and_then(|r| r.strip_prefix('('))
        .and_then(|r| r.strip_suffix(')'));
    if let Some(inside) = inside_opt {
        let lower = inside.to_ascii_lowercase();
        let (high_str, low_str, dir) = if let Some((h, _)) = lower.split_once(" downto ") {
            (
                inside[..h.len()].trim().to_string(),
                inside[h.len() + " downto ".len()..].trim().to_string(),
                RangeDir::Downto,
            )
        } else if let Some((h, _)) = lower.split_once(" to ") {
            (
                inside[..h.len()].trim().to_string(),
                inside[h.len() + " to ".len()..].trim().to_string(),
                RangeDir::To,
            )
        } else if let Some((h, l)) = inside.split_once(':') {
            (
                h.trim().to_string(),
                l.trim().to_string(),
                RangeDir::Downto,
            )
        } else {
            return Err(format!(
                "type '{s}': expected '<h>:<l>' or '<h> downto <l>' inside parentheses"
            ));
        };
        let high = high_str
            .parse::<i64>()
            .map(RangeExpr::Literal)
            .unwrap_or_else(|_| RangeExpr::Expr(high_str));
        let low = low_str
            .parse::<i64>()
            .map(RangeExpr::Literal)
            .unwrap_or_else(|_| RangeExpr::Expr(low_str));
        return Ok(PortType::StdLogicVector(Range { high, low, dir }));
    }
    Ok(PortType::Other(s.to_string()))
}

/// Render a PortType in the brief SV-flavored form used by the top-level
/// mini-editor: `logic` for scalar, `logic[h:l]` for vectors. Ascending
/// (`to`) ranges fall back to the explicit form. Codegen continues to use
/// the strict `port_type_to_vhdl` for actual HDL output.
fn port_type_brief(pt: &PortType) -> String {
    match pt {
        PortType::StdLogic => "logic".to_string(),
        PortType::StdLogicVector(range) => {
            let high = range_bound_str(&range.high);
            let low = range_bound_str(&range.low);
            match range.dir {
                RangeDir::Downto => format!("logic[{high}:{low}]"),
                RangeDir::To => format!("logic[{high} to {low}]"),
            }
        }
        PortType::Record(name) => name.clone(),
        PortType::Other(s) => s.clone(),
    }
}

fn range_bound_str(e: &RangeExpr) -> String {
    match e {
        RangeExpr::Literal(n) => n.to_string(),
        RangeExpr::Expr(s) => s.clone(),
    }
}

fn parse_range_bound(s: &str) -> RangeExpr {
    s.parse::<i64>()
        .map(RangeExpr::Literal)
        .unwrap_or_else(|_| RangeExpr::Expr(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, PortType};

    fn pd(name: &str, dir: Direction, ty: PortType, bundle: Option<&str>) -> PortDef {
        PortDef {
            name: name.to_string(),
            direction: dir,
            port_type: ty,
            bundle: bundle.map(|b| b.to_string()),
        }
    }

    #[test]
    fn resolve_reuses_matching_top_port() {
        let existing = vec![pd(
            "clk",
            Direction::In,
            PortType::StdLogic,
            None,
        )];
        let src = pd("clk", Direction::In, PortType::StdLogic, None);
        let (name, create) = resolve_top_port_name(&existing, &src);
        assert_eq!(name, "clk");
        assert!(!create);
    }

    #[test]
    fn resolve_creates_suffixed_name_on_direction_mismatch() {
        let existing = vec![pd(
            "clk",
            Direction::Out,
            PortType::StdLogic,
            None,
        )];
        let src = pd("clk", Direction::In, PortType::StdLogic, None);
        let (name, create) = resolve_top_port_name(&existing, &src);
        assert_eq!(name, "clk_1");
        assert!(create);
    }

    #[test]
    fn resolve_walks_suffixes_until_unique() {
        let existing = vec![
            pd("clk", Direction::Out, PortType::StdLogic, None),
            pd("clk_1", Direction::Out, PortType::StdLogic, None),
            pd("clk_2", Direction::Out, PortType::StdLogic, None),
        ];
        let src = pd("clk", Direction::In, PortType::StdLogic, None);
        let (name, create) = resolve_top_port_name(&existing, &src);
        assert_eq!(name, "clk_3");
        assert!(create);
    }

    #[test]
    fn resolve_fresh_name_creates_new() {
        let existing = vec![pd("other", Direction::In, PortType::StdLogic, None)];
        let src = pd("clk", Direction::In, PortType::StdLogic, None);
        let (name, create) = resolve_top_port_name(&existing, &src);
        assert_eq!(name, "clk");
        assert!(create);
    }

    #[test]
    fn parse_top_level_minimal() {
        let buf = "clk : in\ndout : out\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.ports.len(), 2);
        assert_eq!(parsed.ports[0].name, "clk");
        assert!(matches!(parsed.ports[0].direction, Direction::In));
        assert!(matches!(parsed.ports[0].port_type, PortType::StdLogic));
    }

    #[test]
    fn parse_top_level_explicit_type() {
        let buf = "bus : out std_logic_vector(7 downto 0)\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.ports.len(), 1);
        match &parsed.ports[0].port_type {
            PortType::StdLogicVector(r) => {
                assert_eq!(r.high, RangeExpr::Literal(7));
                assert_eq!(r.low, RangeExpr::Literal(0));
            }
            other => panic!("expected std_logic_vector, got {other:?}"),
        }
    }

    #[test]
    fn parse_top_level_with_generics() {
        let buf = "WIDTH : integer := 8\n\nclk : in\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.generics.len(), 1);
        assert_eq!(parsed.generics[0].name, "WIDTH");
        assert_eq!(parsed.generics[0].default_value.as_deref(), Some("8"));
        assert_eq!(parsed.ports.len(), 1);
    }

    #[test]
    fn parse_top_level_ignores_decoration() {
        let buf = "entity top is\n  port (\n    clk : in,\n    dout : out\n  );\nend entity;\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.ports.len(), 2);
    }

    #[test]
    fn parse_top_level_strips_comments() {
        let buf = "clk : in -- main system clock\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.ports.len(), 1);
    }

    #[test]
    fn parse_top_level_strips_sv_comments() {
        let buf = "clk : in // main system clock\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.ports.len(), 1);
    }

    #[test]
    fn parse_top_level_ignores_sv_decoration() {
        let buf = "module top (\n    WIDTH : integer := 8,\n    clk : in logic,\n    dout : out logic[7:0],\n);\nendmodule\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert_eq!(parsed.generics.len(), 1);
        assert_eq!(parsed.generics[0].name, "WIDTH");
        assert_eq!(parsed.ports.len(), 2);
        assert_eq!(parsed.ports[0].name, "clk");
        assert_eq!(parsed.ports[1].name, "dout");
    }

    #[test]
    fn parse_top_level_logic_brief() {
        let buf = "clk : in logic\nbus : out logic[7:0]\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        assert!(matches!(parsed.ports[0].port_type, PortType::StdLogic));
        match &parsed.ports[1].port_type {
            PortType::StdLogicVector(r) => {
                assert_eq!(r.high, RangeExpr::Literal(7));
                assert_eq!(r.low, RangeExpr::Literal(0));
                assert!(matches!(r.dir, RangeDir::Downto));
            }
            other => panic!("expected logic vector, got {other:?}"),
        }
    }

    #[test]
    fn parse_top_level_accepts_legacy_slv_paste() {
        let buf = "bus : out slv(7:0)\n";
        let parsed = parse_top_level_buffer(buf).unwrap();
        match &parsed.ports[0].port_type {
            PortType::StdLogicVector(r) => assert_eq!(r.high, RangeExpr::Literal(7)),
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn port_type_brief_renders_logic() {
        let pt = PortType::StdLogicVector(Range {
            high: RangeExpr::Literal(7),
            low: RangeExpr::Literal(0),
            dir: RangeDir::Downto,
        });
        assert_eq!(port_type_brief(&pt), "logic[7:0]");
        assert_eq!(port_type_brief(&PortType::StdLogic), "logic");
    }
}
