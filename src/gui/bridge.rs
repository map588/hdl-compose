#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        /// Make an `AppState` in Rust. The application makes its `AppState`
        /// in C++. The tests use this function to run the methods of
        /// `AppState` with no window.
        #[cxx_name = "make_unique"]
        fn new_app_state() -> UniquePtr<AppState>;
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
        fn set_instance_position(self: Pin<&mut AppState>, name: &QString, x: f64, y: f64) -> bool;

        #[qinvokable]
        fn instance_pos_x(self: &AppState, index: i32) -> f64;

        #[qinvokable]
        fn instance_pos_y(self: &AppState, index: i32) -> f64;

        #[qinvokable]
        fn set_selected_instance(self: Pin<&mut AppState>, name: &QString);

        #[qinvokable]
        fn selected_instance(self: &AppState) -> QString;

        /// Mini-editor: render the instance's bindings buffer.
        #[qinvokable]
        fn instance_buffer(self: &AppState, instance: &QString) -> QString;

        /// Mini-editor: parse-check a buffer (no mutation) — squiggle lines.
        #[qinvokable]
        fn check_instance_buffer(self: &AppState, buffer: &QString) -> FfiEditorIssues;

        /// Mini-editor: parse + apply a buffer as one batch. On any parse
        /// error nothing is applied and the issues come back.
        #[qinvokable]
        fn commit_instance_buffer(
            self: Pin<&mut AppState>,
            instance: &QString,
            buffer: &QString,
        ) -> FfiEditorIssues;

        /// All RHS completion candidates: top ports + "inst.port" pairs,
        /// excluding the instance being edited.
        #[qinvokable]
        fn editor_rhs_candidates(self: &AppState, editing_instance: &QString) -> Vec<String>;

        /// One instance's port names, for dot-completion.
        #[qinvokable]
        fn editor_port_candidates(self: &AppState, instance: &QString) -> Vec<String>;

        /// Try to wire two pins identified by their canvas keys
        /// ("inst.port" / "top:name"). All compatibility rules, driver/load
        /// resolution, and multi-load semantics live Rust-side; the canvas
        /// only reports the outcome (flash + tooltip on error, stay armed
        /// on sticky).
        #[qinvokable]
        fn connect_pins(self: Pin<&mut AppState>, a: &QString, b: &QString) -> FfiConnectResult;

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
        fn consumer_slice(self: &AppState, instance: &QString, port: &QString) -> QString;

        #[qinvokable]
        fn clear_port_map_entry(
            self: Pin<&mut AppState>,
            instance: &QString,
            port: &QString,
        ) -> bool;

        #[qinvokable]
        fn port_map_entry(self: &AppState, instance: &QString, port: &QString) -> QString;

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
        fn module_generic_name(self: &AppState, instance_index: i32, generic_index: i32)
        -> QString;

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
        fn manual_bundle_name(self: &AppState, instance_index: i32, bundle_index: i32) -> QString;

        #[qinvokable]
        fn manual_bundle_port_count(self: &AppState, instance_index: i32, bundle_index: i32)
        -> i32;

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
        fn remove_top_port(self: Pin<&mut AppState>, name: &QString) -> bool;

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
        fn instance_port_name(self: &AppState, instance_index: i32, port_index: i32) -> QString;

        #[qinvokable]
        fn instance_port_direction(self: &AppState, instance_index: i32, port_index: i32) -> i32;

        #[qinvokable]
        fn instance_port_width(self: &AppState, instance_index: i32, port_index: i32) -> i32;

        #[qinvokable]
        fn instance_port_bundle(self: &AppState, instance_index: i32, port_index: i32) -> QString;

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

        /// The identity of the net of a wire. Two wires have the same value
        /// when, and only when, they are on one net of the schematic.
        #[qinvokable]
        fn wire_net(self: &AppState, index: i32) -> QString;

        #[qinvokable]
        fn set_alias(self: Pin<&mut AppState>, net_key: &QString, alias: &QString) -> bool;

        /// Current alias for a net key (empty if none). Slice suffixes are
        /// normalized away, matching set_alias.
        #[qinvokable]
        fn net_alias(self: &AppState, net_key: &QString) -> QString;

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

        /// The board file in the toolchain config of the project. Empty
        /// when the config names no board.
        #[qinvokable]
        fn project_board(self: &AppState) -> QString;

        /// Store the board file in the toolchain config of the project.
        /// On failure this returns false and `last_error` has the reason.
        #[qinvokable]
        fn set_project_board(self: Pin<&mut AppState>, board: &QString) -> bool;

        #[qinvokable]
        fn validation_error_count(self: &AppState) -> i32;

        #[qinvokable]
        fn validation_warning_count(self: &AppState) -> i32;

        /// All current diagnostics, formatted, errors first.
        #[qinvokable]
        fn validation_messages(self: &AppState) -> Vec<String>;

        /// Highest issue severity attached to a pin key ("inst.port" or
        /// "top:name"): 0 = none, 1 = warning, 2 = error.
        #[qinvokable]
        fn pin_issue_level(self: &AppState, key: &QString) -> i32;

        /// Message behind pin_issue_level (empty when clean).
        #[qinvokable]
        fn pin_issue_message(self: &AppState, key: &QString) -> QString;

        /// True when `name` is a collapsed group rendered as a block.
        #[qinvokable]
        fn is_group_block(self: &AppState, name: &QString) -> bool;

        /// Innermost group containing instance `name` ("" when ungrouped).
        #[qinvokable]
        fn instance_group(self: &AppState, name: &QString) -> QString;

        /// Create a collapsed group from newline-separated instance names.
        #[qinvokable]
        fn create_group(self: Pin<&mut AppState>, name: &QString, members: &QString) -> bool;

        /// Dissolve a group: members stay, child groups reparent.
        #[qinvokable]
        fn remove_group(self: Pin<&mut AppState>, name: &QString) -> bool;

        #[qinvokable]
        fn set_group_collapsed(self: Pin<&mut AppState>, name: &QString, collapsed: bool) -> bool;

        #[qsignal]
        fn groups_changed(self: Pin<&mut AppState>);

        /// Number of groups (collapsed or not).
        #[qinvokable]
        fn group_count(self: &AppState) -> i32;

        #[qinvokable]
        fn group_name(self: &AppState, index: i32) -> QString;

        /// True when the group renders as an expanded hull (not collapsed,
        /// no collapsed ancestor).
        #[qinvokable]
        fn group_expanded_visible(self: &AppState, index: i32) -> bool;

        /// Newline-joined canvas item names inside an expanded group's hull:
        /// visible member instances plus collapsed descendant group blocks.
        #[qinvokable]
        fn group_hull_items(self: &AppState, name: &QString) -> QString;

        /// Non-empty when this pin is on a net exposed as a group boundary
        /// port: "exposed as '<port>' on group '<group>'".
        #[qinvokable]
        fn pin_exposed_note(self: &AppState, key: &QString) -> QString;

        /// For a collapsed group block's pin ("group.port"): the internal
        /// pins behind the boundary port, e.g. "from u_pulse.count".
        #[qinvokable]
        fn pin_origin_note(self: &AppState, key: &QString) -> QString;

        /// Direction of any canvas pin key: 0 in, 1 out, 2 inout, -1 unknown.
        #[qinvokable]
        fn pin_direction_code(self: &AppState, key: &QString) -> i32;

        /// Position lock: locked instances only move by manual drag.
        #[qinvokable]
        fn instance_locked(self: &AppState, name: &QString) -> bool;

        #[qinvokable]
        fn set_instance_locked(self: Pin<&mut AppState>, name: &QString, locked: bool) -> bool;

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

    // --- Wire routing FFI ----------------------------------------------
    //
    // Pure geometry; the canvas resolves pin positions / module rects and
    // passes them in, Rust plans the routes (see src/routing.rs — that's
    // where the logic and its property tests live).

    struct FfiEndpoint {
        x: f64,
        y: f64,
        exits_right: bool,
        col: i32,
    }

    struct FfiNet {
        driver: FfiEndpoint,
        loads: Vec<FfiEndpoint>,
    }

    struct FfiRect {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
    }

    struct FfiPoint {
        x: f64,
        y: f64,
    }

    struct FfiWire {
        points: Vec<FfiPoint>,
    }

    struct FfiDot {
        x: f64,
        y: f64,
        /// Index into the input `nets` (per-net dot coloring).
        net: i32,
    }

    struct FfiRouteResult {
        /// One polyline per load, flattened net-major in input order.
        wires: Vec<FfiWire>,
        dots: Vec<FfiDot>,
    }

    struct FfiRouteParams {
        column_pitch: f64,
        lane_step: f64,
        stub_min: f64,
    }

    /// Outcome of connect_pins. `error` is empty for silent refusals
    /// (e.g. unparseable key) — the canvas flashes only when it has text.
    struct FfiConnectResult {
        committed: bool,
        /// Multi-load self-net created: WireTool stays armed for fan-out.
        sticky: bool,
        error: String,
    }

    /// Parse problems in a mini-editor buffer: 0-based line indices (for
    /// the squiggle highlighter) with parallel messages.
    struct FfiEditorIssues {
        lines: Vec<i32>,
        messages: Vec<String>,
    }

    /// One module in a column for push-and-shove placement.
    struct FfiColItem {
        id: i32,
        col: i32,
        top: f64,
        height: f64,
        fixed: bool,
    }

    /// A module the legalizer moved: apply `new_top` and persist it.
    struct FfiColMove {
        id: i32,
        new_top: f64,
    }

    /// One block for layout optimization.
    struct FfiPlaceNode {
        id: i32,
        height: f64,
    }

    /// A wire between pins; `*_dy` = pin offset from its block's top.
    /// `directed` = `from` electrically drives `to`.
    struct FfiPlaceEdge {
        from: i32,
        from_dy: f64,
        to: i32,
        to_dy: f64,
        directed: bool,
    }

    /// Optimized slot for one block.
    struct FfiPlaceResult {
        id: i32,
        col: i32,
        y: f64,
    }

    /// Completion context at the cursor. kind: 0 none, 1 RHS, 2 dot-port.
    struct FfiCompletionContext {
        kind: i32,
        prefix: String,
        instance: String,
    }

    extern "Rust" {
        fn plan_routes_ffi(
            nets: Vec<FfiNet>,
            obstacles: Vec<FfiRect>,
            params: FfiRouteParams,
        ) -> FfiRouteResult;

        /// Push-and-shove: fixed items stay, others shift minimally per
        /// column (see routing::legalize_columns).
        fn legalize_columns_ffi(items: Vec<FfiColItem>, gap: f64) -> Vec<FfiColMove>;

        /// Wire-length-minimizing layout (see routing::optimize_positions).
        /// `max_cols` > 0 caps the column count.
        fn optimize_positions_ffi(
            nodes: Vec<FfiPlaceNode>,
            edges: Vec<FfiPlaceEdge>,
            gap: f64,
            max_cols: i32,
        ) -> Vec<FfiPlaceResult>;

        /// Mini-editor grammar (pure; see src/gui/editor.rs).
        fn completion_context_ffi(line_before_cursor: &str) -> FfiCompletionContext;
    }
}

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::SystemTime;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

use crate::codegen;
use crate::gui::editor;
use crate::project;
use crate::routing;
use crate::schematic::Diagnostic;
use crate::types::{
    Direction, Group, Instance, Language, ModuleDef, NetRef, PortDef, PortType, Range, RangeDir,
    RangeExpr, Schematic, SliceExpr,
};

/// One entry in the canvas view: a real instance or a collapsed group
/// rendered as a synthetic block.
#[derive(Clone, Copy)]
enum CanvasRef {
    Inst(usize),
    Group(usize),
}

/// A group is hidden entirely when any ancestor group is collapsed.
fn group_hidden_by_ancestor(s: &Schematic, g: &Group) -> bool {
    let mut cur = g.parent.as_deref();
    let mut hops = 0;
    while let Some(pname) = cur {
        let Some(p) = s.groups.iter().find(|x| x.name == pname) else {
            return false;
        };
        if p.collapsed {
            return true;
        }
        cur = p.parent.as_deref();
        hops += 1;
        if hops > 64 {
            return false; // cycle guard; validate_groups reports it
        }
    }
    false
}

#[derive(Default)]
pub struct AppStateRust {
    // Qt properties (auto getters/setters/signals)
    project_name: QString,
    dirty: bool,
    // Internal state — not directly exposed as properties
    schematic: Option<Schematic>,
    current_path: Option<PathBuf>,
    last_error: String,
    // What the loader reported for the open project. The loader can change
    // the model (it removes a reference to an instance that is not in the
    // file), so these stay in the diagnostics until the next save.
    load_warnings: Vec<String>,
    library: Vec<ModuleDef>,
    diagnostics: Vec<Diagnostic>,
    selected_instance: String,
    // Flattened wire list cache, one wire for each port_map entry.
    wires: Vec<Wire>,
    // Per-pin issue markers for canvas highlighting, keyed like wires
    // ("inst.port" / "top:name"): (severity 1|2, message). Rebuilt with
    // diagnostics on every validate.
    pin_issues: HashMap<String, (i32, String)>,
    // Pins on nets exposed as a group boundary port, keyed "inst.port" →
    // human note. Rebuilt with every validate; innermost group wins.
    pin_exposed: HashMap<String, String>,
    // Collapsed-group block ports, keyed "group.port" → "from <inside pins>"
    // so the block's tooltips name the internal origin. Rebuilt with
    // pin_exposed.
    pin_origin: HashMap<String, String>,
    // Undo/redo: JSON snapshots of `schematic` taken before each mutation.
    // Capped at UNDO_STACK_LIMIT entries.
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    // Parsed-module cache keyed by source path, invalidated when the file's
    // mtime changes. Without it every mutation re-parses the whole library
    // from disk (rebuild_library_and_validate runs per mutation).
    parse_cache: HashMap<PathBuf, ParseCacheEntry>,
    // Mutation batching: depth > 0 while inside begin_batch/end_batch.
    // Batched mutators skip per-entry validate/signal; end_batch does each
    // once. The first change in a batch takes the one snapshot of the batch.
    batch_depth: u32,
    batch_changed: bool,
    // The model from before the last change, until it is known that the
    // change is one. See `push_snapshot`.
    pending_snapshot: Option<String>,
    // True when the open batch has its undo snapshot.
    batch_recorded: bool,
    // True while Rust sends a signal that makes the view build the canvas
    // again (`project_loaded`, `groups_changed`). See `rebuild_view`.
    view_rebuild: bool,
}

/// One wire of the canvas: one port_map entry.
struct Wire {
    /// The pin that the entry references: "top:<name>" or "<inst>.<port>",
    /// with an optional `[h:l]` slice. This is not always the driver.
    source: String,
    /// The pin that has the entry: always "<inst>.<port>".
    target: String,
    /// Bus width (1 for scalar, N for vector); -1 unknown.
    width: i32,
    /// The identity of the net (see `net_key`).
    net: String,
}

/// A key that identifies a net among the nets of one schematic. The canvas
/// uses it for the color and the highlight of a net, so that it shows the
/// same nets as the generated HDL. A wire alone does not give the net: an
/// entry can reference a pin that is not the driver.
///
/// The key is the one driver of the net, so that the color of a net stays
/// while its loads change. A net with no driver, or with more than one,
/// takes its first pin.
fn net_key(net: &crate::nets::Net) -> String {
    match net.drivers.as_slice() {
        [driver] => driver.to_key(),
        _ => net.members.first().map(NetRef::to_key).unwrap_or_default(),
    }
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
                        ParseCacheEntry {
                            mtime: mt,
                            modules: defs.clone(),
                            error: None,
                        },
                    );
                }
                modules.extend(defs);
            }
            Err(e) => {
                let msg = e.to_string();
                if let Some(mt) = mtime {
                    cache.insert(
                        path.clone(),
                        ParseCacheEntry {
                            mtime: mt,
                            modules: Vec::new(),
                            error: Some(msg.clone()),
                        },
                    );
                }
                errors.push((path.clone(), msg));
            }
        }
    }
    (modules, errors)
}

// --- Routing FFI wrappers (logic lives in crate::routing) -------------------

fn plan_routes_ffi(
    nets: Vec<qobject::FfiNet>,
    obstacles: Vec<qobject::FfiRect>,
    params: qobject::FfiRouteParams,
) -> qobject::FfiRouteResult {
    let conv_ep = |e: &qobject::FfiEndpoint| routing::Endpoint {
        x: e.x,
        y: e.y,
        exits_right: e.exits_right,
        col: e.col,
    };
    let nets_r: Vec<routing::Net> = nets
        .iter()
        .map(|n| routing::Net {
            driver: conv_ep(&n.driver),
            loads: n.loads.iter().map(conv_ep).collect(),
        })
        .collect();
    let obstacles_r: Vec<routing::Rect> = obstacles.iter().map(conv_rect).collect();
    let p = routing::Params {
        column_pitch: params.column_pitch,
        lane_step: params.lane_step,
        stub_min: params.stub_min,
    };

    let result = routing::plan_routes(&nets_r, &obstacles_r, p);
    qobject::FfiRouteResult {
        wires: result
            .wires
            .into_iter()
            .map(|poly| qobject::FfiWire {
                points: poly
                    .into_iter()
                    .map(|pt| qobject::FfiPoint { x: pt.x, y: pt.y })
                    .collect(),
            })
            .collect(),
        dots: result
            .dots
            .into_iter()
            .map(|d| qobject::FfiDot {
                x: d.x,
                y: d.y,
                net: d.net as i32,
            })
            .collect(),
    }
}

fn conv_rect(r: &qobject::FfiRect) -> routing::Rect {
    routing::Rect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}

// --- connect_pins helpers ----------------------------------------------------

struct PinInfo {
    is_top: bool,
    instance: String,
    port: String,
    direction: i32,
    width: i32,
}

fn refuse(error: &str) -> qobject::FfiConnectResult {
    qobject::FfiConnectResult {
        committed: false,
        sticky: false,
        error: error.to_string(),
    }
}

fn done(committed: bool, sticky: bool) -> qobject::FfiConnectResult {
    qobject::FfiConnectResult {
        committed,
        sticky: committed && sticky,
        error: String::new(),
    }
}

fn legalize_columns_ffi(items: Vec<qobject::FfiColItem>, gap: f64) -> Vec<qobject::FfiColMove> {
    let rust_items: Vec<routing::ColItem> = items
        .into_iter()
        .map(|i| routing::ColItem {
            id: i.id,
            col: i.col,
            top: i.top,
            height: i.height,
            fixed: i.fixed,
        })
        .collect();
    routing::legalize_columns(&rust_items, gap)
        .into_iter()
        .map(|(id, new_top)| qobject::FfiColMove { id, new_top })
        .collect()
}

fn optimize_positions_ffi(
    nodes: Vec<qobject::FfiPlaceNode>,
    edges: Vec<qobject::FfiPlaceEdge>,
    gap: f64,
    max_cols: i32,
) -> Vec<qobject::FfiPlaceResult> {
    let ns: Vec<routing::PlaceNode> = nodes
        .into_iter()
        .map(|n| routing::PlaceNode {
            id: n.id,
            height: n.height,
        })
        .collect();
    let es: Vec<routing::PlaceEdge> = edges
        .into_iter()
        .map(|e| routing::PlaceEdge {
            from: e.from,
            from_dy: e.from_dy,
            to: e.to,
            to_dy: e.to_dy,
            directed: e.directed,
        })
        .collect();
    routing::optimize_positions(&ns, &es, gap, max_cols.max(0) as usize)
        .into_iter()
        .map(|(id, col, y)| qobject::FfiPlaceResult { id, col, y })
        .collect()
}

// --- Mini-editor FFI helpers -------------------------------------------------

fn editor_issues(errors: &[(usize, String)]) -> qobject::FfiEditorIssues {
    qobject::FfiEditorIssues {
        lines: errors.iter().map(|(l, _)| *l as i32).collect(),
        messages: errors.iter().map(|(_, m)| m.clone()).collect(),
    }
}

fn completion_context_ffi(line_before_cursor: &str) -> qobject::FfiCompletionContext {
    let ctx = editor::completion_context(line_before_cursor);
    qobject::FfiCompletionContext {
        kind: ctx.kind,
        prefix: ctx.prefix,
        instance: ctx.instance,
    }
}

const UNDO_STACK_LIMIT: usize = 100;

impl qobject::AppState {
    fn record_error(mut self: Pin<&mut Self>, msg: impl Into<String>) {
        self.as_mut().rust_mut().get_mut().last_error = msg.into();
    }

    /// Existence pre-check so mutators can refuse BEFORE snapshotting —
    /// a snapshot taken for an op that then fails leaves a junk undo entry
    /// (Cmd+Z appears to do nothing).
    fn has_instance(&self, name: &str) -> bool {
        self.rust()
            .schematic
            .as_ref()
            .is_some_and(|s| s.instances.iter().any(|i| i.name == name))
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

    /// Open a batch: the changes until `end_batch` are one undo step, with
    /// one validation and one signal. Batches nest; only the outer pair
    /// acts. A batch with no change leaves the history as it is.
    pub fn begin_batch(mut self: Pin<&mut Self>) {
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
            m.batch_recorded = false;
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

    // === Read helpers: index-based lookups into schematic + library ===
    //
    // Canvas indices address the VIEW: visible instances (not inside a
    // collapsed group) followed by visible collapsed-group blocks. A
    // collapsed group renders as a synthetic instance whose ports are the
    // derived boundary ports, so the whole canvas machinery works unchanged.

    fn canvas_refs(&self) -> Vec<CanvasRef> {
        let Some(s) = self.rust().schematic.as_ref() else {
            return Vec::new();
        };
        let mut hidden: HashSet<String> = HashSet::new();
        for g in s.groups.iter().filter(|g| g.collapsed) {
            hidden.extend(crate::groups::transitive_members(s, &g.name));
        }
        let mut out: Vec<CanvasRef> = Vec::new();
        for (i, inst) in s.instances.iter().enumerate() {
            if !hidden.contains(&inst.name) {
                out.push(CanvasRef::Inst(i));
            }
        }
        for (i, g) in s.groups.iter().enumerate() {
            if g.collapsed && !group_hidden_by_ancestor(s, g) {
                out.push(CanvasRef::Group(i));
            }
        }
        out
    }

    fn canvas_ref_at(&self, index: i32) -> Option<CanvasRef> {
        self.canvas_refs().get(index as usize).copied()
    }

    fn canvas_instance_at(&self, index: i32) -> Option<&Instance> {
        match self.canvas_ref_at(index)? {
            CanvasRef::Inst(i) => self.rust().schematic.as_ref()?.instances.get(i),
            CanvasRef::Group(_) => None,
        }
    }

    fn canvas_group_at(&self, index: i32) -> Option<&Group> {
        match self.canvas_ref_at(index)? {
            CanvasRef::Group(i) => self.rust().schematic.as_ref()?.groups.get(i),
            CanvasRef::Inst(_) => None,
        }
    }

    /// Derived boundary ports of a group, as plain PortDefs.
    fn group_ports(&self, group_name: &str) -> Vec<PortDef> {
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else {
            return Vec::new();
        };
        let members = crate::groups::transitive_members(s, group_name);
        crate::groups::derive_boundary(s, &r.library, &members)
            .0
            .into_iter()
            .map(|p| PortDef {
                name: p.name,
                direction: p.direction,
                port_type: p.port_type,
                bundle: None,
            })
            .collect()
    }

    /// Port at a canvas index — owned, because group ports are synthesized.
    fn canvas_port_at(&self, instance_index: i32, port_index: i32) -> Option<PortDef> {
        match self.canvas_ref_at(instance_index)? {
            CanvasRef::Inst(_) => self.instance_port_at(instance_index, port_index).cloned(),
            CanvasRef::Group(i) => {
                let g = self.rust().schematic.as_ref()?.groups.get(i)?;
                self.group_ports(&g.name)
                    .into_iter()
                    .nth(port_index as usize)
            }
        }
    }

    fn instance_module_ports(&self, instance_index: i32) -> Option<&[PortDef]> {
        let r = self.rust();
        let inst = self.canvas_instance_at(instance_index)?;
        let module = r.library.iter().find(|m| m.name == inst.module_ref)?;
        Some(&module.ports)
    }

    fn instance_module_def(&self, instance_index: i32) -> Option<&ModuleDef> {
        let r = self.rust();
        let inst = self.canvas_instance_at(instance_index)?;
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

    // === Library resolution, validation, wire cache ===

    fn rebuild_library_and_validate(mut self: Pin<&mut Self>) {
        let paths: Vec<PathBuf> = match self.as_ref().rust().schematic.as_ref() {
            Some(s) => s.library_paths.clone(),
            None => return,
        };
        let (library, lib_errors) = {
            let cache = &mut self.as_mut().rust_mut().get_mut().parse_cache;
            resolve_library_cached(cache, &paths)
        };
        // A source can change on disk between two calls. Compare the ports
        // of each module with the library of the last call: a connection to
        // a changed port goes away and its instance gets the dirty flag.
        // The result is the same for a reload and for an edit that finds
        // the change.
        let newly_dirty = {
            let m = self.as_mut().rust_mut().get_mut();
            match m.schematic.as_mut() {
                Some(s) => s.apply_library_update(&m.library, &library),
                None => Vec::new(),
            }
        };
        if !newly_dirty.is_empty() {
            self.as_mut().rust_mut().get_mut().dirty = true;
            self.as_mut().set_dirty(true);
            self.as_mut().record_error(format!(
                "library re-parse dropped connections on: {}",
                newly_dirty.join(", ")
            ));
        }
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
        for warning in &self.as_ref().rust().load_warnings {
            diagnostics.push(Diagnostic::warning(format!("load: {warning}")));
        }
        // Surface the most recent library error to the status bar.
        if let Some(d) = diagnostics
            .iter()
            .find(|d| d.is_error() && d.message.starts_with("library: "))
        {
            let msg = d.message.clone();
            self.as_mut().record_error(msg);
        }
        let (pin_issues, (pin_exposed, pin_origin)) = match self.as_ref().rust().schematic.as_ref()
        {
            Some(s) => (
                compute_pin_issues(s, &library, &diagnostics),
                compute_pin_exposed(s, &library),
            ),
            None => (HashMap::new(), (HashMap::new(), HashMap::new())),
        };
        {
            let m = self.as_mut().rust_mut().get_mut();
            m.library = library;
            m.diagnostics = diagnostics;
            m.pin_issues = pin_issues;
            m.pin_exposed = pin_exposed;
            m.pin_origin = pin_origin;
        }
        self.as_mut().rebuild_wires();
        self.as_mut().library_changed();
        self.as_mut().validation_changed();
    }

    fn rebuild_wires(mut self: Pin<&mut Self>) {
        let mut entries: Vec<(String, String, String, i32, String)> = Vec::new();
        // Collapsed groups: pins hidden inside render at the group block's
        // boundary-port pin instead; fully internal wires disappear.
        let mut hidden: HashSet<String> = HashSet::new();
        let mut remap: HashMap<String, String> = HashMap::new();
        {
            let r = self.as_ref();
            let Some(s) = r.rust().schematic.as_ref() else {
                return;
            };
            let lib = &r.rust().library;
            let nets = crate::nets::resolve_nets(s, lib);
            for g in s
                .groups
                .iter()
                .filter(|g| g.collapsed && !group_hidden_by_ancestor(s, g))
            {
                let members = crate::groups::transitive_members(s, &g.name);
                let (_, pfp) = crate::groups::derive_boundary(s, lib, &members);
                for (pin, port) in pfp {
                    if let NetRef::InstancePort(i, _) = &pin
                        && members.contains(i)
                    {
                        remap.insert(pin.to_key(), format!("{}.{}", g.name, port));
                    }
                }
                hidden.extend(members);
            }
            for inst in &s.instances {
                let module = lib.iter().find(|m| m.name == inst.module_ref);
                for (port, net) in &inst.port_map {
                    if let Some(driver) = net {
                        // Constant ties render as a pin badge, not a wire.
                        if matches!(driver, NetRef::Constant(_)) {
                            continue;
                        }
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
                        // The net of the pin, also when a collapsed group
                        // hides the pin and the wire goes to the group block.
                        let pin = NetRef::InstancePort(inst.name.clone(), port.clone());
                        let net = nets
                            .net_for(&pin)
                            .map(net_key)
                            .unwrap_or_else(|| driver.base().to_key());
                        entries.push((inst.name.clone(), port.clone(), driver.to_key(), w, net));
                    }
                }
            }
        }
        entries.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        // Mutual references (A.x => B.y and B.y => A.x) describe one wire —
        // dedup on the unordered pair of base pin keys.
        let mut seen_pairs: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        let wires: Vec<Wire> = entries
            .into_iter()
            .filter_map(|(inst, port, driver, width, net)| {
                let mut target = format!("{inst}.{port}");
                // Self-reference: used only as a net identity marker for
                // multi-load undriven signals. Don't render as a wire.
                if driver == target {
                    return None;
                }
                let mut driver = driver;
                // Remap hidden endpoints onto their group block pins.
                if hidden.contains(&inst) {
                    target = remap.get(&target)?.clone();
                }
                let driver_base = NetRef::from_key(&driver)
                    .map(|r| r.base().to_key())
                    .unwrap_or_else(|| driver.clone());
                if let Some(dinst) = NetRef::from_key(&driver)
                    .as_ref()
                    .and_then(|r| r.instance_name())
                    && hidden.contains(dinst)
                {
                    // Slice suffix drops with the remap — the group pin
                    // carries the whole boundary net.
                    driver = remap.get(&driver_base)?.clone();
                }
                if driver == target {
                    return None; // both ends inside the same group block
                }
                let driver_base = NetRef::from_key(&driver)
                    .map(|r| r.base().to_key())
                    .unwrap_or_else(|| driver.clone());
                let pair = if driver_base < target {
                    (driver_base, target.clone())
                } else {
                    (target.clone(), driver_base)
                };
                if !seen_pairs.insert(pair) {
                    return None;
                }
                Some(Wire {
                    source: driver,
                    target,
                    width,
                    net,
                })
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
                // The file is now the model: the warnings of the loader no
                // longer apply.
                let had_warnings = !std::mem::take(&mut m.load_warnings).is_empty();
                self.as_mut().refresh_project_display();
                if had_warnings {
                    self.as_mut().rebuild_library_and_validate();
                }
                self.as_mut().project_saved();
                true
            }
            Err(e) => {
                self.as_mut().record_error(e.to_string());
                false
            }
        }
    }

    // === Project lifecycle: new / open / save / generate ===

    // --- File ops ---
    pub fn open_project(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        match project::load_project(&path_buf) {
            Ok((schematic, warnings)) => {
                {
                    let m = self.as_mut().rust_mut().get_mut();
                    m.load_warnings = warnings;
                    m.schematic = Some(schematic);
                    m.current_path = Some(path_buf);
                    m.dirty = false;
                    m.last_error.clear();
                    // The library of the last project is not an earlier
                    // library of this one: no comparison of ports.
                    m.library.clear();
                }
                self.as_mut().clear_undo_stacks();
                self.as_mut().refresh_project_display();
                self.as_mut().rebuild_library_and_validate();
                self.as_mut().rebuild_view(|state| state.project_loaded());
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

        // Same entry point as the CLI: a grouped project produces one file
        // for each group in addition to the top-level file.
        let design = match codegen::generate_design(&schematic, &library) {
            Ok(d) => d,
            Err(codegen::DesignError::Top(codegen::CodegenError::ValidationErrors(errs))) => {
                let mut msg = String::from("validation errors prevent codegen:");
                for d in &errs {
                    msg.push_str("\n  ");
                    msg.push_str(&d.to_string());
                }
                self.as_mut().record_error(msg);
                return false;
            }
            Err(codegen::DesignError::Top(codegen::CodegenError::DirtyInstances(names))) => {
                self.as_mut().record_error(format!(
                    "dirty instances present (source re-parse dropped connections). Reconnect: {}",
                    names.join(", ")
                ));
                return false;
            }
            Err(codegen::DesignError::GroupDiagnostics(diags)) => {
                let mut msg = String::from("group validation errors prevent codegen:");
                for d in &diags {
                    msg.push_str("\n  ");
                    msg.push_str(&d.to_string());
                }
                self.as_mut().record_error(msg);
                return false;
            }
            Err(e @ codegen::DesignError::Group { .. }) => {
                self.as_mut().record_error(e.to_string());
                return false;
            }
        };

        let path_buf = PathBuf::from(path.to_string());
        match design.write_to(&path_buf) {
            Ok(_) => {
                self.as_mut().rust_mut().get_mut().last_error.clear();
                true
            }
            Err(e) => {
                self.as_mut()
                    .record_error(format!("write {}: {e}", path_buf.display()));
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

    // === Top-level entity buffer (mini editor, top-level mode) ===

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
        self.as_mut().rebuild_view(|state| state.project_loaded());
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
            m.load_warnings.clear();
            m.library.clear();
            m.diagnostics.clear();
        }
        self.as_mut().clear_undo_stacks();
        self.as_mut().refresh_project_display();
        self.as_mut().rebuild_view(|state| state.project_loaded());
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
        self.canvas_refs().len() as i32
    }

    pub fn instance_name(&self, index: i32) -> QString {
        self.canvas_instance_at(index)
            .map(|i| QString::from(&i.name))
            .or_else(|| self.canvas_group_at(index).map(|g| QString::from(&g.name)))
            .unwrap_or_default()
    }

    pub fn instance_module(&self, index: i32) -> QString {
        self.canvas_instance_at(index)
            .map(|i| QString::from(&i.module_ref))
            .or_else(|| {
                self.canvas_group_at(index)
                    .map(|_| QString::from("(group)"))
            })
            .unwrap_or_default()
    }

    // === Instances: add / remove / rename / position / selection ===

    // --- Instance mutation ---
    pub fn add_instance(mut self: Pin<&mut Self>, name: &QString, module: &QString) -> bool {
        let name_s = name.to_string();
        let module_s = module.to_string();
        if self.has_instance(&name_s) {
            self.as_mut()
                .record_error(format!("duplicate instance name: {name_s}"));
            return false;
        }
        // Batch-aware so drop (add + initial position) is one undo step.
        // Signals still fire immediately — the canvas must create the item
        // before the position arrives.
        self.as_mut().push_snapshot();
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
        if !self.has_instance(&old_s) {
            self.as_mut()
                .record_error(format!("instance not found: {old_s}"));
            return false;
        }
        if self.has_instance(&new_s) {
            self.as_mut()
                .record_error(format!("duplicate instance name: {new_s}"));
            return false;
        }
        self.as_mut().push_snapshot();
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
        let Some(s) = self.rust().schematic.as_ref() else {
            return false;
        };
        s.instances
            .get(index as usize)
            .map(|i| i.dirty)
            .unwrap_or(false)
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

    pub fn set_instance_position(mut self: Pin<&mut Self>, name: &QString, x: f64, y: f64) -> bool {
        let name_s = name.to_string();
        let (is_inst, is_group) = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .map(|s| {
                (
                    s.instances.iter().any(|i| i.name == name_s),
                    s.groups.iter().any(|g| g.name == name_s),
                )
            })
            .unwrap_or((false, false));
        if !is_inst && !is_group {
            return false;
        }
        // Snapshot so a completed move is undoable. Called once per drag
        // release / drop — never per drag tick.
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            if is_inst {
                if let Some(inst) = s.get_instance_mut(&name_s) {
                    inst.position = (x as f32, y as f32);
                }
            } else if let Some(g) = s.groups.iter_mut().find(|g| g.name == name_s) {
                g.position = (x as f32, y as f32);
            }
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().instance_moved(QString::from(&name_s), x, y);
        true
    }

    pub fn instance_pos_x(&self, index: i32) -> f64 {
        self.canvas_instance_at(index)
            .map(|i| i.position.0 as f64)
            .or_else(|| self.canvas_group_at(index).map(|g| g.position.0 as f64))
            .unwrap_or(0.0)
    }

    pub fn instance_pos_y(&self, index: i32) -> f64 {
        self.canvas_instance_at(index)
            .map(|i| i.position.1 as f64)
            .or_else(|| self.canvas_group_at(index).map(|g| g.position.1 as f64))
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
        let Some(inst) = self.canvas_instance_at(index) else {
            return QString::default();
        };
        let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
            return QString::default();
        };
        QString::from(&module.source_path.to_string_lossy().into_owned())
    }

    pub fn remove_instance(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        if !self.has_instance(&name_s) {
            self.as_mut()
                .record_error(format!("instance not found: {name_s}"));
            return false;
        }
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

    // === Mini-editor buffer (grammar in src/gui/editor.rs) ===

    pub fn instance_buffer(&self, instance: &QString) -> QString {
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else {
            return QString::default();
        };
        QString::from(&editor::render_instance_buffer(
            s,
            &r.library,
            &instance.to_string(),
        ))
    }

    pub fn check_instance_buffer(&self, buffer: &QString) -> qobject::FfiEditorIssues {
        editor_issues(&editor::parse_instance_buffer(&buffer.to_string()).errors)
    }

    pub fn commit_instance_buffer(
        mut self: Pin<&mut Self>,
        instance: &QString,
        buffer: &QString,
    ) -> qobject::FfiEditorIssues {
        let parsed = editor::parse_instance_buffer(&buffer.to_string());
        if !parsed.errors.is_empty() {
            // Refuse without mutating; squiggles + status bar tell the user.
            return editor_issues(&parsed.errors);
        }
        if instance.to_string().is_empty() || self.instance_index(instance) < 0 {
            return editor_issues(&[]);
        }
        let mut late_errors: Vec<(usize, String)> = Vec::new();
        // The commit clears the dirty flag of the instance. The clear is in
        // the batch, so one undo brings back the connections and the flag.
        let was_dirty = self.instance_is_dirty_name(instance);
        let any =
            was_dirty || !parsed.generic_commits.is_empty() || !parsed.port_commits.is_empty();
        if any {
            self.as_mut().begin_batch();
        }
        for (g, v) in &parsed.generic_commits {
            self.as_mut()
                .set_generic_map_entry(instance, &QString::from(g), &QString::from(v));
        }
        for (lhs, rhs) in &parsed.port_commits {
            let Some((port, slice)) = editor::split_consumer_slice(lhs) else {
                late_errors.push((0, format!("could not parse port slice in '{lhs}'")));
                continue;
            };
            let port_q = QString::from(&port);
            self.as_mut()
                .set_port_map_entry(instance, &port_q, &QString::from(rhs));
            match slice {
                Some((high, low)) => {
                    self.as_mut()
                        .set_consumer_slice(instance, &port_q, high, low);
                }
                None => {
                    self.as_mut().clear_consumer_slice(instance, &port_q);
                }
            }
        }
        if was_dirty {
            self.as_mut().clear_instance_dirty(instance);
        }
        if any {
            self.as_mut().end_batch();
        }
        editor_issues(&late_errors)
    }

    pub fn editor_rhs_candidates(&self, editing_instance: &QString) -> Vec<String> {
        let editing = editing_instance.to_string();
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else {
            return Vec::new();
        };
        let mut out: Vec<String> = s.top_ports.iter().map(|p| p.name.clone()).collect();
        for inst in &s.instances {
            if inst.name == editing {
                continue;
            }
            if let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) {
                for p in &module.ports {
                    out.push(format!("{}.{}", inst.name, p.name));
                }
            }
        }
        out.sort();
        out
    }

    pub fn editor_port_candidates(&self, instance: &QString) -> Vec<String> {
        let inst_name = instance.to_string();
        let r = self.rust();
        let Some(s) = r.schematic.as_ref() else {
            return Vec::new();
        };
        let Some(inst) = s.instances.iter().find(|i| i.name == inst_name) else {
            return Vec::new();
        };
        let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
            return Vec::new();
        };
        let mut out: Vec<String> = module.ports.iter().map(|p| p.name.clone()).collect();
        out.sort();
        out
    }

    // === Port map, slices, aliases ===

    /// Resolve a canvas pin key to the facts connect_pins needs.
    /// Direction codes 0=in 1=out 2=inout; widths follow the
    /// instance_port_width / top_port_width conventions (0 scalar, N
    /// vector, -1 unresolved).
    fn pin_info(&self, key: &str) -> Option<PinInfo> {
        let r = self.rust();
        let s = r.schematic.as_ref()?;
        if let Some(name) = key.strip_prefix("top:") {
            let p = s.top_ports.iter().find(|p| p.name == name)?;
            return Some(PinInfo {
                is_top: true,
                instance: String::new(),
                port: name.to_string(),
                direction: direction_code(&p.direction),
                width: port_type_width(&p.port_type),
            });
        }
        let (inst_name, port_name) = key.split_once('.')?;
        if let Some(inst) = s.instances.iter().find(|i| i.name == inst_name) {
            let module = r.library.iter().find(|m| m.name == inst.module_ref)?;
            let p = module.ports.iter().find(|p| p.name == port_name)?;
            return Some(PinInfo {
                is_top: false,
                instance: inst_name.to_string(),
                port: port_name.to_string(),
                direction: direction_code(&p.direction),
                width: resolve_port_width(&p.port_type, &module.generics, &inst.generic_map),
            });
        }
        // Collapsed-group block pin: resolvable for hover/width display, but
        // connect_pins refuses group pins (wire inside the group instead).
        if s.groups.iter().any(|g| g.name == inst_name && g.collapsed) {
            let ports = self.group_ports(inst_name);
            let p = ports.iter().find(|p| p.name == port_name)?;
            return Some(PinInfo {
                is_top: false,
                instance: inst_name.to_string(),
                port: port_name.to_string(),
                direction: direction_code(&p.direction),
                width: port_type_width(&p.port_type),
            });
        }
        None
    }

    pub fn connect_pins(
        mut self: Pin<&mut Self>,
        a: &QString,
        b: &QString,
    ) -> qobject::FfiConnectResult {
        let a_key = a.to_string();
        let b_key = b.to_string();
        if a_key == b_key {
            return refuse("cannot connect pin to itself");
        }
        if a_key.is_empty() || b_key.is_empty() {
            return refuse("cannot wire a bundle header; expand it and wire a member port");
        }
        let Some(pa) = self.pin_info(&a_key) else {
            return refuse("");
        };
        let Some(pb) = self.pin_info(&b_key) else {
            return refuse("");
        };
        // Collapsed-group pins are derived, not stored — wiring through them
        // would silently vanish on expand.
        let is_group_pin = |p: &PinInfo| !p.is_top && !self.has_instance(&p.instance);
        if is_group_pin(&pa) || is_group_pin(&pb) {
            return refuse("expand the group to edit its wiring");
        }

        // Compatibility: one driver per net; scalar/vector and widths match.
        if !pa.is_top && !pb.is_top && pa.direction == 1 && pb.direction == 1 {
            return refuse("output-to-output: only one driver per net allowed");
        }
        let norm = |w: i32| if w == 1 { 0 } else { w };
        let (sw, dw) = (norm(pa.width), norm(pb.width));
        if sw == 0 && dw == -1 {
            return refuse("type mismatch: scalar cannot drive vector");
        }
        if sw == -1 && dw == 0 {
            return refuse("type mismatch: vector cannot drive scalar");
        }
        if sw >= 0 && dw >= 0 && sw != dw {
            let label = |w: i32| {
                if w == 0 {
                    "scalar".to_string()
                } else {
                    w.to_string()
                }
            };
            return refuse(&format!("width mismatch: {} vs {}", label(sw), label(dw)));
        }

        if pa.is_top && pb.is_top {
            return refuse("cannot wire two top-level ports");
        }
        if pa.is_top || pb.is_top {
            let (top, pin) = if pa.is_top { (&pa, &pb) } else { (&pb, &pa) };
            let committed = self.as_mut().set_port_map_entry(
                &QString::from(&pin.instance),
                &QString::from(&pin.port),
                &QString::from(&top.port),
            );
            return done(committed, false);
        }

        if pa.direction == 0 && pb.direction == 0 {
            return self.connect_multi_load(&pa, &pb);
        }

        let can_drive = |d: i32| d == 1 || d == 2;
        let (driver, load) = if can_drive(pa.direction) {
            (&pa, &pb)
        } else {
            (&pb, &pa)
        };
        // A 1-bit vector driving a scalar gets an implicit [0] slice.
        if driver.width == 1 && load.width == 0 && !driver.is_top {
            let committed = self.as_mut().set_port_map_entry_slice(
                &QString::from(&load.instance),
                &QString::from(&load.port),
                &QString::from(&driver.instance),
                &QString::from(&driver.port),
                0,
                0,
            );
            return done(committed, false);
        }
        let rhs = if driver.is_top {
            driver.port.clone()
        } else {
            format!("{}.{}", driver.instance, driver.port)
        };
        let committed = self.as_mut().set_port_map_entry(
            &QString::from(&load.instance),
            &QString::from(&load.port),
            &QString::from(&rhs),
        );
        done(committed, false)
    }

    /// Input-to-input wiring: join two undriven (or differently-driven)
    /// loads onto one net. If exactly one side is already driven, extend
    /// that net. Creating a fresh self-named net returns sticky=true so the
    /// wire tool stays armed for further fan-out.
    fn connect_multi_load(
        mut self: Pin<&mut Self>,
        a: &PinInfo,
        b: &PinInfo,
    ) -> qobject::FfiConnectResult {
        let a_rhs = self
            .port_map_entry(&QString::from(&a.instance), &QString::from(&a.port))
            .to_string();
        let b_rhs = self
            .port_map_entry(&QString::from(&b.instance), &QString::from(&b.port))
            .to_string();
        let a_driven = !a_rhs.is_empty();
        let b_driven = !b_rhs.is_empty();

        if a_driven != b_driven {
            let (rhs, target) = if a_driven { (a_rhs, b) } else { (b_rhs, a) };
            let committed = self.as_mut().set_port_map_entry(
                &QString::from(&target.instance),
                &QString::from(&target.port),
                &QString::from(&rhs),
            );
            return done(committed, false);
        }
        if a_driven && b_driven && a_rhs == b_rhs {
            return done(true, false); // already on the same net
        }
        // Root a fresh net at `a`'s own key and hang both pins on it.
        // One batch = one undo step for the pair.
        let rhs = format!("{}.{}", a.instance, a.port);
        self.as_mut().begin_batch();
        let ok_a = self.as_mut().set_port_map_entry(
            &QString::from(&a.instance),
            &QString::from(&a.port),
            &QString::from(&rhs),
        );
        let ok_b = self.as_mut().set_port_map_entry(
            &QString::from(&b.instance),
            &QString::from(&b.port),
            &QString::from(&rhs),
        );
        self.as_mut().end_batch();
        done(ok_a && ok_b, ok_a && ok_b)
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
        if !self.has_instance(&inst) {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().push_snapshot();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s
                .set_port_map_entry(&inst, port_s.clone(), parsed)
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
        if !self.has_instance(&inst) {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().push_snapshot();
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
        if !self.has_instance(&inst) {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(i) = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .schematic
            .as_mut()
            .and_then(|s| s.get_instance_mut(&inst))
        {
            i.consumer_slices.insert(port_s.clone(), slice);
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
        // Refuse before snapshotting: clearing a slice that isn't set must
        // not eat an undo step.
        let has_slice = self
            .rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.iter().find(|i| i.name == inst))
            .is_some_and(|i| i.consumer_slices.contains_key(&port_s));
        if !has_slice {
            return true;
        }
        self.as_mut().push_snapshot();
        if let Some(i) = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .schematic
            .as_mut()
            .and_then(|s| s.get_instance_mut(&inst))
        {
            i.consumer_slices.remove(&port_s);
        }
        self.as_mut().finish_mutation(Some((&inst, &port_s)));
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

        // Phase 1: gather the source port def and compute the resolved top-port
        // name. Collect any error as a string first so we release the immutable
        // borrow on `self` before calling `record_error`. Snapshot only after
        // this succeeds so a refused promote leaves no junk undo entry.
        let lookup: Result<(PortDef, String, bool), String> = {
            let this = self.as_ref();
            let r = this.rust();
            match r.schematic.as_ref() {
                None => Err("no project loaded".to_string()),
                Some(schematic) => match schematic.instances.iter().find(|i| i.name == inst_s) {
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
                },
            }
        };
        let (port_def, resolved_name, need_create) = match lookup {
            Ok(v) => v,
            Err(e) => {
                self.as_mut().record_error(e);
                return QString::default();
            }
        };
        self.as_mut().push_snapshot();

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
            if let Err(e) = s.set_port_map_entry(&inst_s, port_s.clone(), Some(new_top)) {
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
        self.as_mut().rebuild_view(|state| state.project_loaded());
        QString::from(&resolved_name)
    }

    /// Delete a top-level port and clear any instance connections that drove
    /// or were driven by it. Undoable; no-op (returns false) if absent.
    pub fn remove_top_port(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        // Existence check before the snapshot so a miss leaves no undo entry.
        {
            let this = self.as_ref();
            let r = this.rust();
            let exists = r
                .schematic
                .as_ref()
                .is_some_and(|s| s.top_ports.iter().any(|p| p.name == name_s));
            if !exists {
                return false;
            }
        }
        self.as_mut().push_snapshot();
        {
            let s = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .schematic
                .as_mut()
                .expect("checked above");
            s.top_ports.retain(|p| p.name != name_s);
            // Disconnect any port_map entry that referenced this top port.
            let target = NetRef::TopPort(name_s.clone());
            for inst in s.instances.iter_mut() {
                for entry in inst.port_map.values_mut() {
                    if entry.as_ref().is_some_and(|net| net.base() == target) {
                        *entry = None;
                    }
                }
            }
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().rebuild_view(|state| state.project_loaded());
        true
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
        if !self.has_instance(&inst_s) {
            self.as_mut()
                .record_error(format!("instance not found: {inst_s}"));
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(inst) = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .schematic
            .as_mut()
            .and_then(|s| s.get_instance_mut(&inst_s))
        {
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
        // Refuse before snapshotting so a miss leaves no junk undo entry.
        let exists = self
            .rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.iter().find(|i| i.name == inst_s))
            .is_some_and(|i| i.manual_bundles.contains_key(&name_s));
        if !exists {
            self.as_mut()
                .record_error(format!("bundle not found: {inst_s}/{name_s}"));
            return false;
        }
        self.as_mut().push_snapshot();
        let removed = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .schematic
            .as_mut()
            .and_then(|s| s.get_instance_mut(&inst_s))
            .is_some_and(|inst| inst.manual_bundles.remove(&name_s).is_some());
        if removed {
            self.as_mut().rust_mut().get_mut().dirty = true;
            self.as_mut().set_dirty(true);
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().port_map_changed_bulk();
        }
        removed
    }

    pub fn manual_bundle_count(&self, instance_index: i32) -> i32 {
        self.canvas_instance_at(instance_index)
            .map(|i| i.manual_bundles.len() as i32)
            .unwrap_or(0)
    }

    fn manual_bundle_at(
        &self,
        instance_index: i32,
        bundle_index: i32,
    ) -> Option<(&String, &Vec<String>)> {
        let inst = self.canvas_instance_at(instance_index)?;
        // Deterministic order: sort keys so repeated invokable calls align.
        let mut keys: Vec<&String> = inst.manual_bundles.keys().collect();
        keys.sort();
        let key = keys.get(bundle_index as usize)?;
        let ports = inst.manual_bundles.get(*key)?;
        Some((key, ports))
    }

    pub fn manual_bundle_name(&self, instance_index: i32, bundle_index: i32) -> QString {
        self.manual_bundle_at(instance_index, bundle_index)
            .map(|(k, _)| QString::from(k))
            .unwrap_or_default()
    }

    pub fn manual_bundle_port_count(&self, instance_index: i32, bundle_index: i32) -> i32 {
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

        // Phase 1: gather the set of (port_name, top_port_name) pairs to wire,
        // reading from the current schematic + library. All reads; snapshot
        // waits until we know there is something to apply.
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
                    if let Some(Some(_)) = inst.port_map.get(&p.name) {
                        return None;
                    }
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
        self.as_mut().push_snapshot();

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

    // === Library paths, reload, dirty flags ===

    pub fn reload_library(mut self: Pin<&mut Self>) -> bool {
        if self.as_ref().rust().schematic.is_none() {
            return false;
        }
        // Explicit refresh must bypass the mtime check (e.g. sub-second
        // saves, or tools that preserve mtime), so drop the parse cache.
        self.as_mut().rust_mut().get_mut().parse_cache.clear();
        let paths: Vec<PathBuf> = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .map(|s| s.library_paths.clone())
            .unwrap_or_default();
        // Parsing goes through the (just-cleared) cache so the rebuild
        // below reuses it instead of parsing the whole library a second
        // time.
        let new_library: Vec<ModuleDef> = {
            let cache = &mut self.as_mut().rust_mut().get_mut().parse_cache;
            resolve_library_cached(cache, &paths).0
        };
        // The file watcher calls this for each save of a source. Only a
        // reload that changes an instance is an edit; each other reload
        // must leave the undo history as it is.
        let changes_an_instance = {
            let r = self.as_ref();
            let m = r.rust();
            m.schematic.as_ref().is_some_and(|s| {
                !s.clone()
                    .apply_library_update(&m.library, &new_library)
                    .is_empty()
            })
        };
        if changes_an_instance {
            self.as_mut().push_snapshot();
        }
        // This does the comparison and removes the stale connections.
        self.as_mut().rebuild_library_and_validate();
        // Full canvas rebuild so updated widths / generics / port lists reflect.
        self.as_mut().rebuild_view(|state| state.project_loaded());
        true
    }

    /// Clear the dirty flag on an instance. User signals they've reviewed
    /// the dropped-port fallout from a library re-parse.
    pub fn clear_instance_dirty(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        if !self.instance_is_dirty_name(name) {
            return false;
        }
        self.as_mut().push_snapshot();
        let was_dirty = {
            let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() else {
                return false;
            };
            s.clear_instance_dirty(&name_s)
        };
        if was_dirty {
            self.as_mut().rebuild_library_and_validate();
            self.as_mut().rebuild_view(|state| state.project_loaded());
        }
        was_dirty
    }

    // === Undo / redo (snapshot-based; see ARCHITECTURE.md) ===

    /// Snapshot the current schematic for undo. Each method that changes
    /// the model calls this before the change. No-op when no project is
    /// loaded.
    ///
    /// The snapshot is pending first. It becomes an undo step, and removes
    /// the redo history, only when the model is different after the call
    /// (`settle_snapshot`). A call that changes nothing (a name that is in
    /// use, a value that is the same) thus leaves the history as it is, and
    /// no method needs a check of its own for that.
    ///
    /// A batch is one undo step: only the first change in a batch takes a
    /// snapshot. While the view builds the canvas again (`rebuild_view`), no
    /// change takes a snapshot.
    fn push_snapshot(mut self: Pin<&mut Self>) {
        {
            let m = self.as_ref();
            let m = m.rust();
            if m.view_rebuild || (m.batch_depth > 0 && m.batch_recorded) {
                return;
            }
        }
        // The change before this one is complete.
        self.as_mut().settle_snapshot();
        let Some(snap) = self.snapshot_text() else {
            return;
        };
        let m = self.as_mut().rust_mut().get_mut();
        m.pending_snapshot = Some(snap);
        m.batch_recorded = m.batch_depth > 0;
    }

    /// The model as the text of a snapshot. The text is a function of the
    /// model only (the maps have sorted keys), so two texts are equal when,
    /// and only when, the models are equal.
    fn snapshot_text(&self) -> Option<String> {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok())
    }

    /// True when the model changed after the pending snapshot.
    fn pending_changed(&self) -> bool {
        match &self.rust().pending_snapshot {
            Some(pending) => self.snapshot_text().as_ref() != Some(pending),
            None => false,
        }
    }

    /// Make the pending snapshot an undo step when the model changed after
    /// it. A new undo step removes the redo history. Each function that
    /// reads or changes the two stacks calls this first, or uses
    /// `pending_changed`.
    fn settle_snapshot(mut self: Pin<&mut Self>) {
        let changed = self.pending_changed();
        let m = self.as_mut().rust_mut().get_mut();
        let Some(snap) = m.pending_snapshot.take() else {
            return;
        };
        if !changed {
            return;
        }
        m.undo_stack.push(snap);
        if m.undo_stack.len() > UNDO_STACK_LIMIT {
            m.undo_stack.remove(0);
        }
        m.redo_stack.clear();
    }

    /// Send a signal that makes the view build the canvas again from the
    /// model. In its handler the view puts each block on the grid
    /// (`CanvasLayer::settleAll`). Those moves are a part of the change
    /// that sends the signal, so they take no snapshot: a snapshot there
    /// would remove the redo history after each undo.
    fn rebuild_view(mut self: Pin<&mut Self>, send: impl FnOnce(Pin<&mut Self>)) {
        let outer = std::mem::replace(&mut self.as_mut().rust_mut().get_mut().view_rebuild, true);
        send(self.as_mut());
        self.as_mut().rust_mut().get_mut().view_rebuild = outer;
    }

    fn clear_undo_stacks(mut self: Pin<&mut Self>) {
        let m = self.as_mut().rust_mut().get_mut();
        m.undo_stack.clear();
        m.redo_stack.clear();
        m.pending_snapshot = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.rust().undo_stack.is_empty() || self.pending_changed()
    }

    pub fn can_redo(&self) -> bool {
        // A change after the pending snapshot removes the redo history.
        !self.rust().redo_stack.is_empty() && !self.pending_changed()
    }

    pub fn undo(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().settle_snapshot();
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
        self.as_mut().rebuild_view(|state| state.project_loaded());
        true
    }

    pub fn redo(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().settle_snapshot();
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
        self.as_mut().rebuild_view(|state| state.project_loaded());
        true
    }

    // === Generics + manual bundles ===

    pub fn module_generic_count(&self, instance_index: i32) -> i32 {
        self.instance_module_def(instance_index)
            .map(|m| m.generics.len() as i32)
            .unwrap_or(0)
    }

    pub fn module_generic_name(&self, instance_index: i32, generic_index: i32) -> QString {
        self.instance_module_def(instance_index)
            .and_then(|m| m.generics.get(generic_index as usize))
            .map(|g| QString::from(&g.name))
            .unwrap_or_default()
    }

    pub fn module_generic_default(&self, instance_index: i32, generic_index: i32) -> QString {
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
                    NetRef::InstancePortSlice(_, _, _) | NetRef::TopPortSlice(_, _) => {
                        QString::from(&net.to_key())
                    }
                    // Literal in the form parse_net_rhs accepts.
                    NetRef::Constant(lit) => QString::from(lit),
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
        if !self.has_instance(&inst) {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().push_snapshot();
        let result = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => s
                .set_port_map_entry(&inst, port_s.clone(), None)
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

    pub fn set_generic_map_entry(
        mut self: Pin<&mut Self>,
        instance: &QString,
        generic: &QString,
        expr: &QString,
    ) -> bool {
        let inst = instance.to_string();
        let gen_s = generic.to_string();
        let expr_s = expr.to_string();
        if !self.has_instance(&inst) {
            self.as_mut()
                .record_error(format!("instance not found: {inst}"));
            return false;
        }
        self.as_mut().push_snapshot();
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

    // === Port / top-port / wire metadata served to the canvas ===

    // --- Port metadata ---
    pub fn instance_port_count(&self, instance_index: i32) -> i32 {
        match self.canvas_ref_at(instance_index) {
            Some(CanvasRef::Inst(_)) => self
                .instance_module_ports(instance_index)
                .map(|p| p.len() as i32)
                .unwrap_or(0),
            Some(CanvasRef::Group(i)) => self
                .rust()
                .schematic
                .as_ref()
                .and_then(|s| s.groups.get(i))
                .map(|g| self.group_ports(&g.name).len() as i32)
                .unwrap_or(0),
            None => 0,
        }
    }

    pub fn instance_port_name(&self, instance_index: i32, port_index: i32) -> QString {
        self.canvas_port_at(instance_index, port_index)
            .map(|p| QString::from(&p.name))
            .unwrap_or_default()
    }

    pub fn instance_port_direction(&self, instance_index: i32, port_index: i32) -> i32 {
        self.canvas_port_at(instance_index, port_index)
            .map(|p| direction_code(&p.direction))
            .unwrap_or(-1)
    }

    pub fn instance_port_width(&self, instance_index: i32, port_index: i32) -> i32 {
        if let Some(inst) = self.canvas_instance_at(instance_index) {
            let r = self.rust();
            let Some(module) = r.library.iter().find(|m| m.name == inst.module_ref) else {
                return 0;
            };
            let Some(port) = module.ports.get(port_index as usize) else {
                return 0;
            };
            return resolve_port_width(&port.port_type, &module.generics, &inst.generic_map);
        }
        // Group block: boundary types are already resolved.
        self.canvas_port_at(instance_index, port_index)
            .map(|p| port_type_width(&p.port_type))
            .unwrap_or(0)
    }

    pub fn instance_port_bundle(&self, instance_index: i32, port_index: i32) -> QString {
        self.canvas_port_at(instance_index, port_index)
            .and_then(|p| p.bundle)
            .map(|b| QString::from(&b))
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
            .map(|w| QString::from(&w.source))
            .unwrap_or_default()
    }

    pub fn wire_target(&self, index: i32) -> QString {
        self.rust()
            .wires
            .get(index as usize)
            .map(|w| QString::from(&w.target))
            .unwrap_or_default()
    }

    pub fn wire_width(&self, index: i32) -> i32 {
        self.rust()
            .wires
            .get(index as usize)
            .map(|w| w.width)
            .unwrap_or(0)
    }

    pub fn wire_net(&self, index: i32) -> QString {
        self.rust()
            .wires
            .get(index as usize)
            .map(|w| QString::from(&w.net))
            .unwrap_or_default()
    }

    pub fn set_alias(mut self: Pin<&mut Self>, net_key: &QString, alias: &QString) -> bool {
        let key_s = net_key.to_string();
        let alias_s = alias.to_string();
        // Wire keys off the canvas can carry a slice suffix, but aliases are
        // keyed by the base driver net (signal_name looks up base()).
        let Some(net_ref) = NetRef::from_key(&key_s).map(|n| n.base()) else {
            self.as_mut().record_error("invalid net key");
            return false;
        };
        self.as_mut().push_snapshot();
        // One net has one alias. The key can be each pin of the net, so
        // first remove the alias of each other pin of the net.
        let members = self.net_members(&net_ref);
        let ok = match self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            Some(s) => {
                for member in &members {
                    s.remove_alias(member);
                }
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
        // An alias can make an error (one name on two nets). Validate
        // before the signal; in a batch `end_batch` validates.
        if self.as_ref().rust().batch_depth > 0 {
            self.as_mut().rust_mut().get_mut().batch_changed = true;
        } else {
            self.as_mut().rebuild_library_and_validate();
        }
        self.as_mut().alias_changed(QString::from(&key_s));
        true
    }

    pub fn net_alias(&self, net_key: &QString) -> QString {
        let key_s = net_key.to_string();
        let Some(net_ref) = NetRef::from_key(&key_s).map(|n| n.base()) else {
            return QString::default();
        };
        let Some(s) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        // The alias of the net: it can be on another pin of the net.
        std::iter::once(net_ref.clone())
            .chain(self.net_members(&net_ref))
            .find_map(|pin| s.aliases.get(&pin))
            .map(QString::from)
            .unwrap_or_default()
    }

    /// The pins of the net of `pin`. Empty when the pin has no connection.
    fn net_members(&self, pin: &NetRef) -> Vec<NetRef> {
        let Some(s) = self.rust().schematic.as_ref() else {
            return Vec::new();
        };
        crate::nets::resolve_nets(s, &self.rust().library)
            .net_for(pin)
            .map(|net| net.members.clone())
            .unwrap_or_default()
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
        match self.as_ref().rust().schematic.as_ref() {
            // A path that is in the project already is no change.
            Some(s) if s.library_paths.contains(&path_buf) => return false,
            Some(_) => {}
            None => {
                self.as_mut().record_error("no project loaded");
                return false;
            }
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            s.add_library_path(path_buf);
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

    pub fn project_board(&self) -> QString {
        let Some(project) = self.rust().current_path.as_ref() else {
            return QString::default();
        };
        let dir = project
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        crate::toolchain::ToolchainConfig::load_for(project)
            .ok()
            .and_then(|config| config.board_path(dir))
            .map(|board| QString::from(&board.to_string_lossy().into_owned()))
            .unwrap_or_default()
    }

    pub fn set_project_board(mut self: Pin<&mut Self>, board: &QString) -> bool {
        let Some(project) = self.rust().current_path.clone() else {
            self.as_mut().record_error("save the project first");
            return false;
        };
        let board = PathBuf::from(board.to_string());
        match crate::toolchain::ToolchainConfig::set_board(&project, &board) {
            Ok(()) => true,
            Err(msg) => {
                self.as_mut().record_error(msg);
                false
            }
        }
    }

    pub fn remove_library_path(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_buf = PathBuf::from(path.to_string());
        match self.as_ref().rust().schematic.as_ref() {
            Some(s) if s.library_paths.contains(&path_buf) => {}
            _ => return false,
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            s.remove_library_path(&path_buf);
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        true
    }

    // --- Validation ---
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

    pub fn validation_messages(&self) -> Vec<String> {
        let diags = &self.rust().diagnostics;
        let mut out: Vec<String> = diags
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.to_string())
            .collect();
        out.extend(
            diags
                .iter()
                .filter(|d| !d.is_error())
                .map(|d| d.to_string()),
        );
        out
    }

    pub fn pin_issue_level(&self, key: &QString) -> i32 {
        self.rust()
            .pin_issues
            .get(&key.to_string())
            .map(|(l, _)| *l)
            .unwrap_or(0)
    }

    pub fn pin_issue_message(&self, key: &QString) -> QString {
        self.rust()
            .pin_issues
            .get(&key.to_string())
            .map(|(_, m)| QString::from(m))
            .unwrap_or_default()
    }

    // === Groups ===

    pub fn is_group_block(&self, name: &QString) -> bool {
        let n = name.to_string();
        self.rust()
            .schematic
            .as_ref()
            .is_some_and(|s| s.groups.iter().any(|g| g.name == n && g.collapsed))
    }

    pub fn instance_group(&self, name: &QString) -> QString {
        let n = name.to_string();
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| {
                s.groups
                    .iter()
                    .find(|g| g.members.contains(&n))
                    .map(|g| QString::from(&g.name))
            })
            .unwrap_or_default()
    }

    pub fn create_group(mut self: Pin<&mut Self>, name: &QString, members: &QString) -> bool {
        let name_s = name.to_string().trim().to_string();
        let member_list: Vec<String> = members
            .to_string()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if name_s.is_empty() || member_list.is_empty() {
            self.as_mut().record_error("group needs a name and members");
            return false;
        }
        let err: Option<String> = {
            let this = self.as_ref();
            let r = this.rust();
            match r.schematic.as_ref() {
                None => Some("no project loaded".to_string()),
                Some(s) => {
                    if s.groups.iter().any(|g| g.name == name_s)
                        || s.instances.iter().any(|i| i.name == name_s)
                        || r.library.iter().any(|m| m.name == name_s)
                    {
                        Some(format!("name '{name_s}' is already taken"))
                    } else {
                        member_list.iter().find_map(|m| {
                            if !s.instances.iter().any(|i| i.name == *m) {
                                Some(format!("no such instance: {m}"))
                            } else if s.groups.iter().any(|g| g.members.contains(m)) {
                                Some(format!("'{m}' is already in a group"))
                            } else {
                                None
                            }
                        })
                    }
                }
            }
        };
        if let Some(e) = err {
            self.as_mut().record_error(e);
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            let pts: Vec<(f32, f32)> = s
                .instances
                .iter()
                .filter(|i| member_list.contains(&i.name))
                .map(|i| i.position)
                .collect();
            let n = pts.len().max(1) as f32;
            let centroid = (
                pts.iter().map(|p| p.0).sum::<f32>() / n,
                pts.iter().map(|p| p.1).sum::<f32>() / n,
            );
            s.groups.push(Group {
                name: name_s.clone(),
                members: member_list,
                parent: None,
                collapsed: true,
                position: centroid,
            });
        }
        self.as_mut().finish_group_mutation()
    }

    pub fn remove_group(mut self: Pin<&mut Self>, name: &QString) -> bool {
        let name_s = name.to_string();
        let exists = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .is_some_and(|s| s.groups.iter().any(|g| g.name == name_s));
        if !exists {
            self.as_mut()
                .record_error(format!("no such group: {name_s}"));
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            let parent = s
                .groups
                .iter()
                .find(|g| g.name == name_s)
                .and_then(|g| g.parent.clone());
            for g in s.groups.iter_mut() {
                if g.parent.as_deref() == Some(name_s.as_str()) {
                    g.parent = parent.clone();
                }
            }
            s.groups.retain(|g| g.name != name_s);
        }
        self.as_mut().finish_group_mutation()
    }

    pub fn set_group_collapsed(mut self: Pin<&mut Self>, name: &QString, collapsed: bool) -> bool {
        let name_s = name.to_string();
        let exists = self
            .as_ref()
            .rust()
            .schematic
            .as_ref()
            .is_some_and(|s| s.groups.iter().any(|g| g.name == name_s));
        if !exists {
            self.as_mut()
                .record_error(format!("no such group: {name_s}"));
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut() {
            let members = crate::groups::transitive_members(s, &name_s);
            let pts: Vec<(f32, f32)> = s
                .instances
                .iter()
                .filter(|i| members.contains(&i.name))
                .map(|i| i.position)
                .collect();
            let centroid = if pts.is_empty() {
                (0.0, 0.0)
            } else {
                let n = pts.len() as f32;
                (
                    pts.iter().map(|p| p.0).sum::<f32>() / n,
                    pts.iter().map(|p| p.1).sum::<f32>() / n,
                )
            };
            if collapsed {
                // The block appears where its members were.
                if let Some(g) = s.groups.iter_mut().find(|g| g.name == name_s) {
                    g.position = centroid;
                    g.collapsed = true;
                }
            } else {
                // Members follow wherever the collapsed block was dragged:
                // translate them (and nested collapsed blocks) by the
                // block's displacement from their stored centroid.
                let g_pos = s
                    .groups
                    .iter()
                    .find(|g| g.name == name_s)
                    .map(|g| g.position)
                    .unwrap_or(centroid);
                let delta = (g_pos.0 - centroid.0, g_pos.1 - centroid.1);
                if delta.0.abs() > 0.5 || delta.1.abs() > 0.5 {
                    for inst in s.instances.iter_mut().filter(|i| members.contains(&i.name)) {
                        inst.position.0 += delta.0;
                        inst.position.1 += delta.1;
                    }
                    // Descendant group blocks ride along too.
                    let mut desc: HashSet<String> = HashSet::new();
                    let mut queue = vec![name_s.clone()];
                    while let Some(cur) = queue.pop() {
                        for g in &s.groups {
                            if g.parent.as_deref() == Some(cur.as_str())
                                && desc.insert(g.name.clone())
                            {
                                queue.push(g.name.clone());
                            }
                        }
                    }
                    for g in s.groups.iter_mut().filter(|g| desc.contains(&g.name)) {
                        g.position.0 += delta.0;
                        g.position.1 += delta.1;
                    }
                }
                if let Some(g) = s.groups.iter_mut().find(|g| g.name == name_s) {
                    g.collapsed = false;
                }
            }
        }
        self.as_mut().finish_group_mutation()
    }

    /// Shared tail for group mutators: dirty + revalidate + notify.
    fn finish_group_mutation(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        self.as_mut().rebuild_library_and_validate();
        self.as_mut().rebuild_view(|state| state.groups_changed());
        true
    }

    pub fn group_count(&self) -> i32 {
        self.rust()
            .schematic
            .as_ref()
            .map(|s| s.groups.len() as i32)
            .unwrap_or(0)
    }

    pub fn group_name(&self, index: i32) -> QString {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.groups.get(index as usize))
            .map(|g| QString::from(&g.name))
            .unwrap_or_default()
    }

    pub fn group_expanded_visible(&self, index: i32) -> bool {
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| {
                s.groups
                    .get(index as usize)
                    .map(|g| !g.collapsed && !group_hidden_by_ancestor(s, g))
            })
            .unwrap_or(false)
    }

    pub fn group_hull_items(&self, name: &QString) -> QString {
        let gname = name.to_string();
        let Some(s) = self.rust().schematic.as_ref() else {
            return QString::default();
        };
        // Visible content of an expanded group: direct member instances,
        // plus each child group either as its collapsed block or (expanded)
        // its own visible content.
        fn collect(s: &Schematic, gname: &str, out: &mut Vec<String>) {
            if let Some(g) = s.groups.iter().find(|g| g.name == gname) {
                out.extend(g.members.iter().cloned());
            }
            for child in s
                .groups
                .iter()
                .filter(|c| c.parent.as_deref() == Some(gname))
            {
                if child.collapsed {
                    out.push(child.name.clone());
                } else {
                    collect(s, &child.name, out);
                }
            }
        }
        let mut out = Vec::new();
        collect(s, &gname, &mut out);
        QString::from(&out.join("\n"))
    }

    pub fn pin_exposed_note(&self, key: &QString) -> QString {
        self.rust()
            .pin_exposed
            .get(&key.to_string())
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn pin_origin_note(&self, key: &QString) -> QString {
        self.rust()
            .pin_origin
            .get(&key.to_string())
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn pin_direction_code(&self, key: &QString) -> i32 {
        self.pin_info(&key.to_string())
            .map(|p| p.direction)
            .unwrap_or(-1)
    }

    pub fn instance_locked(&self, name: &QString) -> bool {
        let n = name.to_string();
        self.rust()
            .schematic
            .as_ref()
            .and_then(|s| s.instances.iter().find(|i| i.name == n))
            .map(|i| i.locked)
            .unwrap_or(false)
    }

    pub fn set_instance_locked(mut self: Pin<&mut Self>, name: &QString, locked: bool) -> bool {
        let n = name.to_string();
        if !self.has_instance(&n) {
            self.as_mut()
                .record_error(format!("instance not found: {n}"));
            return false;
        }
        self.as_mut().push_snapshot();
        if let Some(s) = self.as_mut().rust_mut().get_mut().schematic.as_mut()
            && let Some(inst) = s.get_instance_mut(&n)
        {
            inst.locked = locked;
        }
        self.as_mut().rust_mut().get_mut().dirty = true;
        self.as_mut().set_dirty(true);
        true
    }
}

/// Map diagnostics onto canvas pin keys for inline highlighting. Instance-
/// scoped diagnostics land on their pin; net-level conditions (no driver,
/// driver conflict) land on every affected member pin.
fn compute_pin_issues(
    s: &Schematic,
    library: &[ModuleDef],
    diagnostics: &[Diagnostic],
) -> HashMap<String, (i32, String)> {
    let mut map: HashMap<String, (i32, String)> = HashMap::new();
    let add = |map: &mut HashMap<String, (i32, String)>, key: String, level: i32, msg: &str| {
        let e = map.entry(key).or_insert((0, String::new()));
        if level > e.0 {
            *e = (level, msg.to_string());
        }
    };
    for d in diagnostics {
        if let (Some(i), Some(p)) = (&d.instance, &d.port) {
            let level = if d.is_error() { 2 } else { 1 };
            add(&mut map, format!("{i}.{p}"), level, &d.message);
        }
    }
    let lib_map: HashMap<&str, &ModuleDef> = library.iter().map(|m| (m.name.as_str(), m)).collect();
    let nets = crate::nets::resolve_nets(s, library);
    for net in &nets.nets {
        let hard: Vec<&NetRef> = net
            .drivers
            .iter()
            .filter(|r| crate::nets::pin_direction(s, &lib_map, r) != Some(Direction::InOut))
            .collect();
        let conflicting: Vec<&&NetRef> = hard
            .iter()
            .filter(|a| {
                hard.iter()
                    .any(|b| *b != **a && nets.drivers_conflict(s, a, b))
            })
            .collect();
        if !conflicting.is_empty() {
            let msg = format!("net '{}' has multiple drivers", net.name);
            for r in &conflicting {
                add(&mut map, r.to_key(), 2, &msg);
            }
        }
        if net.drivers.is_empty() {
            let msg = format!("net '{}' has no driver", net.name);
            for m in &net.members {
                add(&mut map, m.to_key(), 1, &msg);
            }
        }
    }
    map
}

/// Map member pins onto the group boundary ports they're exposed through
/// (for expanded-view badges) and, inversely, each group port onto the
/// internal pins behind it (for collapsed-block tooltips). Groups processed
/// shallow→deep; the innermost group's note wins on the pin side.
fn compute_pin_exposed(
    s: &Schematic,
    library: &[ModuleDef],
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut origins: HashMap<String, Vec<String>> = HashMap::new();
    let mut out: HashMap<String, String> = HashMap::new();
    let mut groups: Vec<&Group> = s.groups.iter().collect();
    let by_name: HashMap<&str, &Group> = s.groups.iter().map(|g| (g.name.as_str(), g)).collect();
    let depth = |g: &Group| {
        let mut d = 0;
        let mut cur = g;
        while let Some(p) = cur.parent.as_deref().and_then(|p| by_name.get(p)) {
            d += 1;
            cur = p;
            if d > 64 {
                break;
            }
        }
        d
    };
    groups.sort_by_key(|g| depth(g));
    for g in groups {
        let members = crate::groups::transitive_members(s, &g.name);
        let (_, port_for_pin) = crate::groups::derive_boundary(s, library, &members);
        for (pin, port) in port_for_pin {
            if let NetRef::InstancePort(i, _) = &pin
                && members.contains(i)
            {
                out.insert(
                    pin.to_key(),
                    format!("exposed as '{port}' on group '{}'", g.name),
                );
                origins
                    .entry(format!("{}.{port}", g.name))
                    .or_default()
                    .push(pin.to_key());
            }
        }
    }
    let origin_notes = origins
        .into_iter()
        .map(|(k, mut pins)| {
            pins.sort();
            pins.dedup();
            let shown = pins.len().min(3);
            let mut note = format!("from {}", pins[..shown].join(", "));
            if pins.len() > shown {
                note.push_str(&format!(" (+{} more)", pins.len() - shown));
            }
            (k, note)
        })
        .collect();
    (out, origin_notes)
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
pub(super) fn resolve_top_port_name(existing: &[PortDef], src: &PortDef) -> (String, bool) {
    let matches = |tp: &PortDef| {
        tp.direction == src.direction && tp.port_type == src.port_type && tp.bundle == src.bundle
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
            (RangeExpr::Literal(h), RangeExpr::Literal(l)) => ((h - l).abs() as i32 + 1).max(1),
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
    // "open" is case-insensitive, matching the VHDL keyword and the C++
    // editor's check — a literal "OPEN" must not become a net named OPEN.
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("open") {
        return None;
    }
    if is_constant_literal(trimmed) {
        return Some(NetRef::Constant(trimmed.to_string()));
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

use super::editor::is_constant_literal;

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
            (h.trim().to_string(), l.trim().to_string(), RangeDir::Downto)
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

    // --- AppState with no window -------------------------------------------
    //
    // `qobject::new_app_state` makes the object that the C++ side uses. A
    // closure on a signal stands for the handler of the C++ side.

    use cxx::UniquePtr;
    use qobject::AppState;

    fn q(text: &str) -> QString {
        QString::from(text)
    }

    fn vhdl_entity(name: &str, ports: &str) -> String {
        format!(
            "library ieee;\nuse ieee.std_logic_1164.all;\n\nentity {name} is\n  port ({ports});\nend entity;\n\narchitecture rtl of {name} is\nbegin\nend architecture;\n"
        )
    }

    /// A VHDL project with the modules `source` (output `q`) and `sink`
    /// (inputs `d` and `e`) in its library, and an empty history.
    fn state_with_library() -> (tempfile::TempDir, UniquePtr<AppState>) {
        state_with_sink("d : in std_logic; e : in std_logic")
    }

    /// As `state_with_library`, with the given ports of `sink`.
    fn state_with_sink(sink_ports: &str) -> (tempfile::TempDir, UniquePtr<AppState>) {
        let dir = tempfile::tempdir().unwrap();
        let mut state = qobject::new_app_state();
        assert!(state.pin_mut().new_project(&q("top"), 0));
        for (name, ports) in [("source", "q : out std_logic"), ("sink", sink_ports)] {
            let path = dir.path().join(format!("{name}.vhd"));
            std::fs::write(&path, vhdl_entity(name, ports)).unwrap();
            assert!(state.pin_mut().add_library_path(&q(path.to_str().unwrap())));
        }
        // To open a project makes the history empty.
        let project = q(dir.path().join("top.hdlc").to_str().unwrap());
        assert!(state.pin_mut().save_project_as(&project));
        assert!(state.pin_mut().open_project(&project));
        assert!(!state.can_undo() && !state.can_redo());
        (dir, state)
    }

    /// What `CanvasLayer::settleAll` does on the C++ side after it builds
    /// the canvas again: it opens a batch, moves each block that is not on
    /// its place, and closes the batch. Here the place of `u_a` is (0, 100).
    fn settle(mut state: Pin<&mut AppState>) {
        state.as_mut().begin_batch();
        let index = state.instance_index(&q("u_a"));
        if index >= 0 && (state.instance_pos_x(index), state.instance_pos_y(index)) != (0.0, 100.0)
        {
            state.as_mut().set_instance_position(&q("u_a"), 0.0, 100.0);
        }
        state.as_mut().end_batch();
    }

    /// Regression: `undo` sends `project_loaded`, and the view settles the
    /// blocks in its handler. The batch of the settle took a snapshot and
    /// removed the redo history, so redo did nothing and a second undo
    /// restored the same state.
    #[test]
    fn undo_and_redo_go_through_each_edit_while_the_view_settles() {
        let (_dir, mut state) = state_with_library();
        let _view = state.pin_mut().on_project_loaded(settle);
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));

        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 1);
        assert!(state.can_redo(), "the settle removed the redo history");
        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 0);
        assert!(!state.can_undo(), "the settle added an undo step");

        assert!(state.pin_mut().redo());
        assert!(state.pin_mut().redo());
        assert_eq!(state.instance_count(), 2);
        assert!(!state.can_redo());
    }

    /// Regression: a batch took its snapshot when it opened. A command that
    /// changed nothing (a tidy of a tidy canvas) added an undo step and
    /// removed the redo history.
    #[test]
    fn batch_with_no_change_leaves_the_history() {
        let (_dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().undo());

        state.pin_mut().begin_batch();
        state.pin_mut().end_batch();

        assert!(state.can_redo());
        assert!(!state.can_undo());
    }

    /// Regression: `rename_instance` and nine more methods took a snapshot
    /// of their own in an open batch, so one batch was more than one undo
    /// step.
    #[test]
    fn batch_is_one_undo_step() {
        let (_dir, mut state) = state_with_library();
        state.pin_mut().begin_batch();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        assert!(state.pin_mut().rename_instance(&q("u_b"), &q("u_c")));
        state.pin_mut().end_batch();

        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 0);
        assert!(!state.can_undo());
    }

    /// Regression: the view settles the blocks after a group change too.
    /// That settle was an undo step of its own.
    #[test]
    fn group_change_with_its_settle_is_one_undo_step() {
        let (_dir, mut state) = state_with_library();
        let _view = state.pin_mut().on_groups_changed(settle);
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().create_group(&q("g"), &q("u_a")));
        assert_eq!(state.group_count(), 1);

        assert!(state.pin_mut().undo());
        assert_eq!(state.group_count(), 0);
        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 0);
        assert!(!state.can_undo());
    }

    /// Regression: a change of the library paths took no snapshot, so undo
    /// did not revert it.
    #[test]
    fn library_path_changes_are_undo_steps() {
        let (dir, mut state) = state_with_library();
        let extra = dir.path().join("extra.vhd");
        std::fs::write(&extra, vhdl_entity("extra", "x : in std_logic")).unwrap();
        let extra = q(extra.to_str().unwrap());

        assert!(state.pin_mut().add_library_path(&extra));
        assert_eq!(state.library_path_count(), 3);
        assert!(state.pin_mut().undo());
        assert_eq!(state.library_path_count(), 2);
        assert!(state.pin_mut().redo());
        assert_eq!(state.library_path_count(), 3);

        assert!(state.pin_mut().remove_library_path(&extra));
        assert_eq!(state.library_path_count(), 2);
        assert!(state.pin_mut().undo());
        assert_eq!(state.library_path_count(), 3);

        // A path that is in the project already is no change.
        assert!(!state.pin_mut().add_library_path(&extra));
        assert!(state.can_redo());
    }

    /// Make `u_b` dirty: the type of its port `e` changes in the source.
    fn make_sink_dirty(dir: &tempfile::TempDir, state: &mut UniquePtr<AppState>) {
        std::fs::write(
            dir.path().join("sink.vhd"),
            vhdl_entity(
                "sink",
                "d : in std_logic; e : in std_logic_vector(1 downto 0)",
            ),
        )
        .unwrap();
        assert!(state.pin_mut().reload_library());
        assert!(state.instance_is_dirty_name(&q("u_b")));
    }

    /// Regression: `clear_instance_dirty` took no snapshot.
    #[test]
    fn clear_of_the_dirty_flag_is_an_undo_step() {
        let (dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        make_sink_dirty(&dir, &mut state);

        assert!(state.pin_mut().clear_instance_dirty(&q("u_b")));
        assert!(!state.instance_is_dirty_name(&q("u_b")));
        assert!(state.pin_mut().undo());
        assert!(state.instance_is_dirty_name(&q("u_b")));
    }

    /// Regression: `reload_library` took a snapshot on each call. The file
    /// watcher calls it for each save of a source, so a save that changed no
    /// port added an undo step and removed the redo history.
    #[test]
    fn reload_with_no_change_of_an_instance_leaves_the_history() {
        let (_dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        assert!(state.pin_mut().undo());

        assert!(state.pin_mut().reload_library());

        assert!(state.can_redo(), "the reload removed the redo history");
        assert!(state.pin_mut().undo());
        assert!(!state.can_undo(), "the reload added an undo step");
    }

    /// `u_a.q` drives `u_b.e`.
    fn state_with_one_connection() -> (tempfile::TempDir, UniquePtr<AppState>) {
        let (dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        assert!(
            state
                .pin_mut()
                .set_port_map_entry(&q("u_b"), &q("e"), &q("u_a.q"))
        );
        (dir, state)
    }

    fn entry(state: &UniquePtr<AppState>, instance: &str, port: &str) -> String {
        state.port_map_entry(&q(instance), &q(port)).to_string()
    }

    /// A reload that removes a connection is one undo step, and it makes
    /// the project modified. After the undo the old connection is back, and
    /// validation reports it: no flag is necessary for that.
    #[test]
    fn reload_that_changes_an_instance_is_one_undo_step() {
        let (dir, mut state) = state_with_one_connection();
        let project = q(dir.path().join("top.hdlc").to_str().unwrap());
        assert!(state.pin_mut().save_project_as(&project));
        assert!(!*state.dirty());

        make_sink_dirty(&dir, &mut state);
        assert_eq!(entry(&state, "u_b", "e"), "");
        assert!(*state.dirty(), "the reload changed the model");

        assert!(state.pin_mut().undo());
        assert!(!state.instance_is_dirty_name(&q("u_b")));
        assert_eq!(entry(&state, "u_b", "e"), "u_a.q");
        assert!(state.validation_error_count() > 0);
    }

    /// Regression: a change of a source that came in with an edit (the file
    /// watcher did not report it) got no comparison. The connection to the
    /// changed port stayed, and the instance got no flag. A reload gave
    /// another result for the same change.
    #[test]
    fn changed_source_gets_the_same_comparison_with_an_edit_as_with_a_reload() {
        let (dir, mut state) = state_with_one_connection();
        let path = dir.path().join("sink.vhd");
        std::fs::write(
            &path,
            vhdl_entity(
                "sink",
                "d : in std_logic; e : in std_logic_vector(1 downto 0)",
            ),
        )
        .unwrap();
        // The parse cache compares modification times.
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(later)
            .unwrap();

        assert!(state.pin_mut().add_instance(&q("u_c"), &q("source")));

        assert!(state.instance_is_dirty_name(&q("u_b")));
        assert_eq!(entry(&state, "u_b", "e"), "");
        // The comparison is a part of the edit that found the change.
        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 2);
        assert!(!state.instance_is_dirty_name(&q("u_b")));
        assert_eq!(entry(&state, "u_b", "e"), "u_a.q");
    }

    /// The library of one project is not the earlier library of the next
    /// project: a module with the same name and other ports is no change.
    #[test]
    fn open_of_another_project_compares_with_no_earlier_library() {
        let (_dir, mut state) = state_with_one_connection();
        // The other project has a `sink` with another port `e`.
        let (other_dir, mut other) =
            state_with_sink("d : in std_logic; e : in std_logic_vector(1 downto 0)");
        assert!(other.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(other.pin_mut().add_instance(&q("u_b"), &q("sink")));
        assert!(
            other
                .pin_mut()
                .set_port_map_entry(&q("u_b"), &q("d"), &q("u_a.q"))
        );
        assert!(other.pin_mut().save_project());

        let project = other_dir.path().join("top.hdlc");
        assert!(state.pin_mut().open_project(&q(project.to_str().unwrap())));

        assert!(!state.instance_is_dirty_name(&q("u_b")));
        assert_eq!(entry(&state, "u_b", "d"), "u_a.q");
    }

    /// Regression: `open_project` discarded the warnings of the loader.
    /// The loader removes a reference to an instance that is not in the
    /// file, so the model was not the file and the user got no message.
    #[test]
    fn open_shows_the_warnings_of_the_loader_until_the_save() {
        let (dir, mut state) = state_with_one_connection();
        assert!(state.pin_mut().save_project());
        // Remove `u_a` from the file. `u_b.e` then references no instance.
        let path = dir.path().join("top.hdlc");
        let mut file: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        file["instances"]
            .as_array_mut()
            .unwrap()
            .retain(|i| i["name"] != "u_a");
        std::fs::write(&path, file.to_string()).unwrap();
        let cleared = |state: &UniquePtr<AppState>| {
            state
                .validation_messages()
                .iter()
                .any(|m| m.contains("cleared 1 stale"))
        };

        assert!(state.pin_mut().open_project(&q(path.to_str().unwrap())));
        assert!(cleared(&state), "{:?}", state.validation_messages());
        // An edit does not remove the message.
        assert!(state.pin_mut().add_instance(&q("u_c"), &q("source")));
        assert!(cleared(&state));

        // After the save the file is the model.
        assert!(state.pin_mut().save_project());
        assert!(!cleared(&state));
    }

    /// Regression: the canvas took the pin that an entry references as the
    /// net of the wire. A file can reference a load, or reference from the
    /// driver to the load. The canvas then showed one net of the generated
    /// HDL as two nets. `wire_net` gives the net of `resolve_nets`.
    #[test]
    fn wires_of_one_net_have_one_net_key() {
        let (_dir, mut state) = state_with_library();
        for (name, module) in [("u_a", "source"), ("u_b", "sink"), ("u_c", "sink")] {
            assert!(state.pin_mut().add_instance(&q(name), &q(module)));
        }
        let mut connect = |instance: &str, port: &str, to: &str| {
            assert!(
                state
                    .pin_mut()
                    .set_port_map_entry(&q(instance), &q(port), &q(to))
            );
        };
        // One net, written as a chain: the second entry references a load.
        connect("u_b", "d", "u_a.q");
        connect("u_c", "d", "u_b.d");
        // A second net with no driver.
        connect("u_c", "e", "u_b.e");

        let net_of = |target: &str| {
            (0..state.wire_count())
                .find(|&i| state.wire_target(i).to_string() == target)
                .map(|i| state.wire_net(i).to_string())
                .unwrap_or_else(|| panic!("no wire to {target}"))
        };
        // The key of the first net is its driver, as for a net that the
        // canvas itself made.
        assert_eq!(net_of("u_b.d"), "u_a.q");
        assert_eq!(net_of("u_c.d"), "u_a.q");
        assert_ne!(net_of("u_c.e"), "u_a.q");
        // The source of the second wire is a load: the old identity.
        assert_eq!(state.wire_source(1).to_string(), "u_b.d");
    }

    /// Regression: an alias was a property of the pin that the wire
    /// references. Two wires of one net with different such pins gave the
    /// net two aliases, which is an error.
    #[test]
    fn alias_is_a_property_of_the_net() {
        let (_dir, mut state) = state_with_library();
        for (name, module) in [("u_a", "source"), ("u_b", "sink"), ("u_c", "sink")] {
            assert!(state.pin_mut().add_instance(&q(name), &q(module)));
        }
        // One net as a chain: the wires reference `u_a.q` and `u_b.d`.
        for (instance, to) in [("u_b", "u_a.q"), ("u_c", "u_b.d")] {
            assert!(
                state
                    .pin_mut()
                    .set_port_map_entry(&q(instance), &q("d"), &q(to))
            );
        }

        assert!(state.pin_mut().set_alias(&q("u_a.q"), &q("first")));
        assert_eq!(state.net_alias(&q("u_b.d")).to_string(), "first");
        assert!(state.pin_mut().set_alias(&q("u_b.d"), &q("second")));

        assert_eq!(
            state.validation_error_count(),
            0,
            "{:?}",
            state.validation_messages()
        );
        for pin in ["u_a.q", "u_b.d", "u_c.d"] {
            assert_eq!(state.net_alias(&q(pin)).to_string(), "second", "{pin}");
        }
        // An empty alias removes the alias of the net.
        assert!(state.pin_mut().set_alias(&q("u_c.d"), &q("")));
        assert_eq!(state.net_alias(&q("u_a.q")).to_string(), "");
    }

    /// An entry on the driver that references the load is the same net.
    #[test]
    fn wire_from_the_driver_to_the_load_has_the_key_of_the_driver() {
        let (_dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        assert!(
            state
                .pin_mut()
                .set_port_map_entry(&q("u_a"), &q("q"), &q("u_b.d"))
        );
        assert_eq!(state.wire_count(), 1);
        assert_eq!(state.wire_source(0).to_string(), "u_b.d");
        assert_eq!(state.wire_net(0).to_string(), "u_a.q");
    }

    /// The canvas refuses a connection that is an error. The text editor
    /// accepts each line that it can parse. The rule that holds for the two
    /// paths: validation reports each connection that the canvas refuses,
    /// and code generation stops on an error. This test keeps the rule.
    #[test]
    fn validation_reports_each_connection_that_the_canvas_refuses() {
        // (module of `u_x`, pin of `u_x`, the other pin): two outputs, a
        // scalar on a vector, and two vectors of different widths.
        let cases = [
            ("source", "q", "u_a.q"),
            ("sink", "e", "u_a.q"),
            ("sink", "e", "u_w.q"),
        ];
        for (module, port, from) in cases {
            let (dir, mut state) =
                state_with_sink("d : in std_logic; e : in std_logic_vector(1 downto 0)");
            let wide = dir.path().join("wide.vhd");
            std::fs::write(
                &wide,
                vhdl_entity("wide", "q : out std_logic_vector(3 downto 0)"),
            )
            .unwrap();
            assert!(state.pin_mut().add_library_path(&q(wide.to_str().unwrap())));
            for (name, of) in [("u_a", "source"), ("u_w", "wide"), ("u_x", module)] {
                assert!(state.pin_mut().add_instance(&q(name), &q(of)));
            }
            assert_eq!(state.validation_error_count(), 0);

            // The canvas refuses and changes nothing.
            let pin = format!("u_x.{port}");
            let result = state.pin_mut().connect_pins(&q(from), &q(&pin));
            assert!(!result.committed, "{from} -> {pin}");
            assert!(!result.error.is_empty());
            assert_eq!(entry(&state, "u_x", port), "");

            // The text editor accepts the same connection.
            let buffer = q(&format!(
                "u_x : {module}\n  port map (\n    {port} => {from},\n  );\n"
            ));
            let issues = state.pin_mut().commit_instance_buffer(&q("u_x"), &buffer);
            assert!(issues.messages.is_empty(), "{:?}", issues.messages);
            assert_eq!(entry(&state, "u_x", port), from);
            assert!(
                state.validation_error_count() > 0,
                "{from} -> {pin}: {:?}",
                state.validation_messages()
            );
        }
    }

    /// A commit of the editor buffer clears the dirty flag of its instance.
    /// The connections and the flag come back with one undo.
    #[test]
    fn commit_on_a_dirty_instance_is_one_undo_step() {
        let (dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_b"), &q("sink")));
        make_sink_dirty(&dir, &mut state);

        let buffer = q("u_b : sink\n  port map (\n    d => u_a.q,\n  );\n");
        let issues = state.pin_mut().commit_instance_buffer(&q("u_b"), &buffer);
        assert!(issues.messages.is_empty(), "{:?}", issues.messages);
        assert!(!state.instance_is_dirty_name(&q("u_b")));
        assert!(
            !state
                .port_map_entry(&q("u_b"), &q("d"))
                .to_string()
                .is_empty()
        );

        assert!(state.pin_mut().undo());
        assert!(state.instance_is_dirty_name(&q("u_b")));
        assert!(
            state
                .port_map_entry(&q("u_b"), &q("d"))
                .to_string()
                .is_empty()
        );
    }

    /// Regression: `set_alias` did not run validation, so the diagnostics
    /// were those from before the change. One alias on two nets is an error.
    #[test]
    fn alias_change_runs_validation() {
        let (_dir, mut state) = state_with_library();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().add_instance(&q("u_c"), &q("source")));
        assert!(state.pin_mut().set_alias(&q("u_a.q"), &q("data_net")));
        assert_eq!(
            state.validation_error_count(),
            0,
            "{:?}",
            state.validation_messages()
        );

        assert!(state.pin_mut().set_alias(&q("u_c.q"), &q("data_net")));
        let messages = state.validation_messages();
        assert!(
            messages
                .iter()
                .any(|m| m.contains("alias 'data_net' is used for 2 different nets")),
            "{messages:?}"
        );
    }

    /// An alias change in a batch is a part of the one undo step of the
    /// batch.
    #[test]
    fn alias_change_in_a_batch_takes_no_snapshot_of_its_own() {
        let (_dir, mut state) = state_with_library();
        state.pin_mut().begin_batch();
        assert!(state.pin_mut().add_instance(&q("u_a"), &q("source")));
        assert!(state.pin_mut().set_alias(&q("u_a.q"), &q("named")));
        state.pin_mut().end_batch();

        assert!(state.pin_mut().undo());
        assert_eq!(state.instance_count(), 0);
        assert!(!state.can_undo());
    }

    // --- Property: the history of AppState ---------------------------------

    /// One call of a method that the C++ side uses to change the model.
    /// The names come from small sets, so that a call often hits an
    /// instance that exists, and often one that does not.
    #[derive(Debug, Clone)]
    enum Edit {
        AddInstance(&'static str, &'static str),
        RemoveInstance(&'static str),
        RenameInstance(&'static str, &'static str),
        Move(&'static str, i8, i8),
        Lock(&'static str, bool),
        ConnectPins(&'static str, &'static str),
        SetEntry(&'static str, &'static str, &'static str),
        ClearEntry(&'static str, &'static str),
        SetSlice(&'static str, &'static str, i8, i8),
        ClearSlice(&'static str, &'static str),
        SetGeneric(&'static str, &'static str),
        SetAlias(&'static str, &'static str),
        Promote(&'static str, &'static str),
        RemoveTopPort(&'static str),
        MatchByName(&'static str),
        CreateGroup(&'static str, &'static str),
        RemoveGroup(&'static str),
        Collapse(&'static str, bool),
        CommitBuffer(&'static str, &'static str),
        Reload,
    }

    #[derive(Debug, Clone)]
    enum Step {
        One(Edit),
        Batch(Vec<Edit>),
        Undo,
        Redo,
    }

    const INSTANCES: &[&str] = &["u_a", "u_b", "u_c"];
    const MODULES: &[&str] = &["source", "sink", "missing"];
    const PORTS: &[&str] = &["q", "d", "e", "zz"];
    const PINS: &[&str] = &[
        "u_a.q", "u_b.d", "u_b.e", "u_c.d", "u_c.q", "top:q", "u_z.q",
    ];
    const REFERENCES: &[&str] = &[
        "u_a.q", "u_b.d", "u_c.q", "q", "", "'0'", "u_a.q[0]", "u_z.q",
    ];
    const NAMES: &[&str] = &["", "net_a", "net_b"];
    const GROUPS: &[&str] = &["g1", "g2"];
    const MEMBERS: &[&str] = &["u_a", "u_a,u_b", "u_b,u_c", "g1", "u_z"];
    const BUFFERS: &[&str] = &[
        "u_b : sink\n  port map (\n    d => u_a.q,\n  );\n",
        "u_b : sink\n  port map (\n    d => u_a.q,\n    e => u_c.q,\n  );\n",
        "u_b : sink\n  port map (\n  );\n",
    ];

    fn edit() -> impl proptest::strategy::Strategy<Value = Edit> {
        use proptest::prelude::*;
        use proptest::sample::select;
        prop_oneof![
            4 => (select(INSTANCES), select(MODULES)).prop_map(|(i, m)| Edit::AddInstance(i, m)),
            1 => select(INSTANCES).prop_map(Edit::RemoveInstance),
            1 => (select(INSTANCES), select(INSTANCES)).prop_map(|(a, b)| Edit::RenameInstance(a, b)),
            1 => (select(INSTANCES), 0i8..3, 0i8..3).prop_map(|(i, x, y)| Edit::Move(i, x, y)),
            1 => (select(INSTANCES), any::<bool>()).prop_map(|(i, on)| Edit::Lock(i, on)),
            3 => (select(PINS), select(PINS)).prop_map(|(a, b)| Edit::ConnectPins(a, b)),
            3 => (select(INSTANCES), select(PORTS), select(REFERENCES))
                .prop_map(|(i, p, r)| Edit::SetEntry(i, p, r)),
            1 => (select(INSTANCES), select(PORTS)).prop_map(|(i, p)| Edit::ClearEntry(i, p)),
            1 => (select(INSTANCES), select(PORTS), 0i8..3, 0i8..3)
                .prop_map(|(i, p, h, l)| Edit::SetSlice(i, p, h, l)),
            1 => (select(INSTANCES), select(PORTS)).prop_map(|(i, p)| Edit::ClearSlice(i, p)),
            1 => (select(INSTANCES), select(NAMES)).prop_map(|(i, v)| Edit::SetGeneric(i, v)),
            1 => (select(PINS), select(NAMES)).prop_map(|(p, a)| Edit::SetAlias(p, a)),
            1 => (select(INSTANCES), select(PORTS)).prop_map(|(i, p)| Edit::Promote(i, p)),
            1 => select(PORTS).prop_map(Edit::RemoveTopPort),
            1 => select(INSTANCES).prop_map(Edit::MatchByName),
            1 => (select(GROUPS), select(MEMBERS)).prop_map(|(g, m)| Edit::CreateGroup(g, m)),
            1 => select(GROUPS).prop_map(Edit::RemoveGroup),
            1 => (select(GROUPS), any::<bool>()).prop_map(|(g, on)| Edit::Collapse(g, on)),
            1 => (select(INSTANCES), select(BUFFERS)).prop_map(|(i, b)| Edit::CommitBuffer(i, b)),
            1 => Just(Edit::Reload),
        ]
    }

    fn step() -> impl proptest::strategy::Strategy<Value = Step> {
        use proptest::prelude::*;
        prop_oneof![
            8 => edit().prop_map(Step::One),
            2 => proptest::collection::vec(edit(), 0..4).prop_map(Step::Batch),
            2 => Just(Step::Undo),
            1 => Just(Step::Redo),
        ]
    }

    /// The result of each call has no effect on the property, so it is
    /// not used: a call that the bridge refuses is a part of the input.
    fn apply(state: &mut UniquePtr<AppState>, edit: &Edit) {
        let s = state.pin_mut();
        match *edit {
            Edit::AddInstance(i, m) => {
                s.add_instance(&q(i), &q(m));
            }
            Edit::RemoveInstance(i) => {
                s.remove_instance(&q(i));
            }
            Edit::RenameInstance(a, b) => {
                s.rename_instance(&q(a), &q(b));
            }
            Edit::Move(i, x, y) => {
                s.set_instance_position(&q(i), f64::from(x) * 640.0, f64::from(y) * 100.0);
            }
            Edit::Lock(i, on) => {
                s.set_instance_locked(&q(i), on);
            }
            Edit::ConnectPins(a, b) => {
                s.connect_pins(&q(a), &q(b));
            }
            Edit::SetEntry(i, p, r) => {
                s.set_port_map_entry(&q(i), &q(p), &q(r));
            }
            Edit::ClearEntry(i, p) => {
                s.clear_port_map_entry(&q(i), &q(p));
            }
            Edit::SetSlice(i, p, h, l) => {
                s.set_consumer_slice(&q(i), &q(p), i32::from(h), i32::from(l));
            }
            Edit::ClearSlice(i, p) => {
                s.clear_consumer_slice(&q(i), &q(p));
            }
            Edit::SetGeneric(i, v) => {
                s.set_generic_map_entry(&q(i), &q("WIDTH"), &q(v));
            }
            Edit::SetAlias(p, a) => {
                s.set_alias(&q(p), &q(a));
            }
            Edit::Promote(i, p) => {
                s.promote_port_to_top(&q(i), &q(p));
            }
            Edit::RemoveTopPort(p) => {
                s.remove_top_port(&q(p));
            }
            Edit::MatchByName(i) => {
                s.match_by_name(&q(i));
            }
            Edit::CreateGroup(g, m) => {
                s.create_group(&q(g), &q(m));
            }
            Edit::RemoveGroup(g) => {
                s.remove_group(&q(g));
            }
            Edit::Collapse(g, on) => {
                s.set_group_collapsed(&q(g), on);
            }
            Edit::CommitBuffer(i, b) => {
                s.commit_instance_buffer(&q(i), &q(b));
            }
            Edit::Reload => {
                s.reload_library();
            }
        }
    }

    /// The model as a value that does not depend on the order of a map.
    fn model(state: &UniquePtr<AppState>) -> serde_json::Value {
        serde_json::to_value(&state.rust().schematic).unwrap()
    }

    /// The number of undo steps and of redo steps, with the pending
    /// snapshot as `settle_snapshot` will record it.
    fn history(state: &UniquePtr<AppState>) -> (usize, usize) {
        let (undo, redo) = (state.rust().undo_stack.len(), state.rust().redo_stack.len());
        if state.pending_changed() {
            (undo + 1, 0)
        } else {
            (undo, redo)
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(128))]

        /// The history is correct for each sequence of calls:
        ///
        /// - A call or a batch that changes the model adds one undo step
        ///   and removes the redo steps. One undo gives the model from
        ///   before the call, and one redo gives the model after it.
        /// - A call or a batch that does not change the model does not
        ///   change the history.
        /// - Undo to the start gives the first model.
        #[test]
        fn history_follows_the_model(steps in proptest::collection::vec(step(), 0..40)) {
            use proptest::prelude::*;
            let (_dir, mut state) = state_with_library();
            let first = model(&state);
            for step in &steps {
                let before = model(&state);
                let (undo_before, redo_before) = history(&state);
                match step {
                    Step::Undo => {
                        let did = state.pin_mut().undo();
                        prop_assert_eq!(did, undo_before > 0);
                        continue;
                    }
                    Step::Redo => {
                        let did = state.pin_mut().redo();
                        prop_assert_eq!(did, redo_before > 0);
                        continue;
                    }
                    Step::One(edit) => apply(&mut state, edit),
                    Step::Batch(edits) => {
                        state.pin_mut().begin_batch();
                        for edit in edits {
                            apply(&mut state, edit);
                        }
                        state.pin_mut().end_batch();
                    }
                }
                let after = model(&state);
                if after == before {
                    prop_assert_eq!(
                        history(&state), (undo_before, redo_before),
                        "{:?} changed nothing but changed the history", step
                    );
                } else {
                    prop_assert_eq!(
                        history(&state), (undo_before + 1, 0),
                        "{:?} changed the model", step
                    );
                    prop_assert!(state.pin_mut().undo());
                    prop_assert_eq!(&model(&state), &before, "undo of {:?}", step);
                    prop_assert!(state.pin_mut().redo());
                    prop_assert_eq!(&model(&state), &after, "redo of {:?}", step);
                }
            }
            while state.pin_mut().undo() {}
            prop_assert_eq!(model(&state), first);
        }
    }

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
        let existing = vec![pd("clk", Direction::In, PortType::StdLogic, None)];
        let src = pd("clk", Direction::In, PortType::StdLogic, None);
        let (name, create) = resolve_top_port_name(&existing, &src);
        assert_eq!(name, "clk");
        assert!(!create);
    }

    #[test]
    fn resolve_creates_suffixed_name_on_direction_mismatch() {
        let existing = vec![pd("clk", Direction::Out, PortType::StdLogic, None)];
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

    #[test]
    fn parse_net_rhs_open_and_empty() {
        assert_eq!(parse_net_rhs(""), None);
        assert_eq!(parse_net_rhs("   "), None);
        assert_eq!(parse_net_rhs("open"), None);
        assert_eq!(parse_net_rhs("OPEN"), None);
        assert_eq!(parse_net_rhs(" Open "), None);
    }

    #[test]
    fn parse_net_rhs_variants() {
        assert_eq!(parse_net_rhs("clk"), Some(NetRef::TopPort("clk".into())));
        assert_eq!(
            parse_net_rhs("u_a.dout"),
            Some(NetRef::InstancePort("u_a".into(), "dout".into()))
        );
        assert_eq!(
            parse_net_rhs("u_a.dout[3]"),
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "dout".into(),
                SliceExpr::Bit(3)
            ))
        );
        assert_eq!(
            parse_net_rhs("u_a.dout[7:4]"),
            Some(NetRef::InstancePortSlice(
                "u_a".into(),
                "dout".into(),
                SliceExpr::Range { high: 7, low: 4 }
            ))
        );
        assert_eq!(
            parse_net_rhs("bus[0]"),
            Some(NetRef::TopPortSlice("bus".into(), SliceExpr::Bit(0)))
        );
    }

    #[test]
    fn parse_net_rhs_malformed_slices() {
        assert_eq!(parse_net_rhs("u_a.dout[3"), None, "unclosed bracket");
        assert_eq!(parse_net_rhs("u_a.dout[x]"), None, "non-numeric index");
        assert_eq!(parse_net_rhs("u_a.dout[1:x]"), None, "non-numeric bound");
    }

    #[test]
    fn library_cache_parses_and_tracks_mtime() {
        use std::time::{Duration, SystemTime};
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("mod.vhd");
        std::fs::copy("tests/fixtures/counter.vhd", &path).unwrap();
        let paths = vec![path.clone()];
        let mut cache = HashMap::new();

        let (mods, errs) = resolve_library_cached(&mut cache, &paths);
        assert_eq!(errs.len(), 0);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "counter");

        // Unchanged mtime → served from cache (same result).
        let (mods, errs) = resolve_library_cached(&mut cache, &paths);
        assert_eq!((mods.len(), errs.len()), (1, 0));

        // Rewrite with a renamed entity and force the mtime forward — the
        // cache must notice and re-parse.
        let renamed = std::fs::read_to_string("tests/fixtures/counter.vhd")
            .unwrap()
            .replace("counter", "counter2");
        std::fs::write(&path, renamed).unwrap();
        let f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.set_modified(SystemTime::now() + Duration::from_secs(5))
            .unwrap();

        let (mods, errs) = resolve_library_cached(&mut cache, &paths);
        assert_eq!(errs.len(), 0);
        assert_eq!(mods[0].name, "counter2");
    }

    #[test]
    fn library_cache_caches_parse_errors() {
        let dir = tempfile::TempDir::new().unwrap();
        // .v, not .vhd: vhdl_lang skips garbage with diagnostics but no Err;
        // sv-parser fails hard, which is the path this test pins.
        let path = dir.path().join("broken.v");
        std::fs::write(&path, "this is not Verilog at all }{").unwrap();
        let paths = vec![path.clone()];
        let mut cache = HashMap::new();

        let (mods, errs) = resolve_library_cached(&mut cache, &paths);
        assert_eq!((mods.len(), errs.len()), (0, 1));
        // Second call returns the cached error rather than dropping it.
        let (mods, errs) = resolve_library_cached(&mut cache, &paths);
        assert_eq!((mods.len(), errs.len()), (0, 1));
        assert_eq!(errs[0].0, path);
    }

    #[test]
    fn library_cache_drops_removed_paths() {
        let dir = tempfile::TempDir::new().unwrap();
        let a = dir.path().join("a.vhd");
        std::fs::copy("tests/fixtures/counter.vhd", &a).unwrap();
        let mut cache = HashMap::new();

        let (mods, _) = resolve_library_cached(&mut cache, std::slice::from_ref(&a));
        assert_eq!(mods.len(), 1);
        assert!(cache.contains_key(&a));

        // Path removed from the library → its cache entry must not linger.
        let (mods, _) = resolve_library_cached(&mut cache, &[]);
        assert!(mods.is_empty());
        assert!(!cache.contains_key(&a));
    }
}
