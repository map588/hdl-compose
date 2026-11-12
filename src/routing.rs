//! Pure wire-routing geometry for the canvas.
//!
//! Direct port of the channel router that lived in `src/gui/canvas.h`
//! (`planNet` / `allocateLaneX` / `adjustBridgeY` / `resolveClearY`). No Qt
//! types, no scene access — callers resolve pin positions and module rects
//! and pass them in, which is what makes this property-testable.
//!
//! Model: instances snap to a column grid. Wires route through the vertical
//! "gutters" between columns. A net is one driver endpoint plus N load
//! endpoints; each load gets a polyline:
//!
//! * all endpoints share a gutter → driver stub → vertical trunk → load stub
//! * endpoints span gutters → driver stub → driver-gutter drop → horizontal
//!   bridge at a shared Y → load-gutter rise → load stub
//!
//! Junction dots mark fan-out taps so crossing wires aren't misread as
//! connected.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Endpoint {
    pub x: f64,
    pub y: f64,
    /// True when the wire leaves this pin travelling right (right-edge pins
    /// and left-side top-level ports).
    pub exits_right: bool,
    /// Column index of the owning module (or the top port's own column).
    pub col: i32,
}

#[derive(Clone, Debug)]
pub struct Net {
    pub driver: Endpoint,
    pub loads: Vec<Endpoint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[cfg(test)]
impl Rect {
    fn contains_strict(&self, x: f64, y: f64, eps: f64) -> bool {
        x > self.left + eps && x < self.right - eps && y > self.top + eps && y < self.bottom - eps
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub column_pitch: f64,
    pub lane_step: f64,
    pub stub_min: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Dot {
    pub x: f64,
    pub y: f64,
    /// Index into the input `nets` slice (for per-net coloring).
    pub net: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RouteResult {
    /// One polyline per load, flattened net-major in input order.
    pub wires: Vec<Vec<Point>>,
    pub dots: Vec<Dot>,
}

fn gutter_index(col: i32, exits_right: bool) -> i32 {
    if exits_right {
        2 * col + 1
    } else {
        2 * col - 1
    }
}

fn gutter_center_x(idx: i32, pitch: f64) -> f64 {
    idx as f64 / 2.0 * pitch
}

#[derive(Clone, Copy)]
struct GutterInfo {
    safe_min: f64,
    safe_max: f64,
    nets_using: i32,
}

impl Default for GutterInfo {
    fn default() -> Self {
        GutterInfo {
            safe_min: -1e18,
            safe_max: 1e18,
            nets_using: 0,
        }
    }
}

/// Tighten each gutter's safe X window and count how many nets share each
/// gutter (lane density). Two things bound a window:
///
/// * pins: every wire keeps at least `stub_min` of straight stub at its pin;
/// * block bodies: a lane keeps `stub_min` clear of every block in the two
///   columns beside the gutter, at any Y. This is what keeps a lane from
///   running through a block that has no pin facing the gutter.
fn build_gutter_info(nets: &[Net], obstacles: &[Rect], p: Params) -> HashMap<i32, GutterInfo> {
    let mut info: HashMap<i32, GutterInfo> = HashMap::new();
    for net in nets {
        let mut net_gutters: HashSet<i32> = HashSet::new();
        let mut tighten = |e: &Endpoint, net_gutters: &mut HashSet<i32>| {
            let idx = gutter_index(e.col, e.exits_right);
            net_gutters.insert(idx);
            let gi = info.entry(idx).or_default();
            if e.exits_right {
                gi.safe_min = gi.safe_min.max(e.x + p.stub_min);
            } else {
                gi.safe_max = gi.safe_max.min(e.x - p.stub_min);
            }
        };
        tighten(&net.driver, &mut net_gutters);
        for l in &net.loads {
            tighten(l, &mut net_gutters);
        }
        for g in net_gutters {
            info.entry(g).or_default().nets_using += 1;
        }
    }
    for r in obstacles {
        // Blocks are centered on their column.
        let col = ((r.left + r.right) / 2.0 / p.column_pitch).round() as i32;
        if let Some(gi) = info.get_mut(&gutter_index(col, true)) {
            gi.safe_min = gi.safe_min.max(r.right + p.stub_min);
        }
        if let Some(gi) = info.get_mut(&gutter_index(col, false)) {
            gi.safe_max = gi.safe_max.min(r.left - p.stub_min);
        }
    }
    info
}

/// X for lane `slot` of `total` in gutter `idx`, spread evenly around the
/// gutter's usable center. Squeezes the step when the gutter is bounded on
/// both sides so every lane stays inside the safe window.
fn lane_x(idx: i32, slot: usize, total: usize, info: &HashMap<i32, GutterInfo>, p: Params) -> f64 {
    let gi = info.get(&idx).copied().unwrap_or_default();
    let natural_x = gutter_center_x(idx, p.column_pitch);
    let bounded_left = gi.safe_min > -1e17;
    let bounded_right = gi.safe_max < 1e17;
    let offset = |step: f64| (slot as f64 - (total.saturating_sub(1)) as f64 / 2.0) * step;

    if bounded_left && bounded_right {
        // An over-constrained gutter (pins on both sides demand more room
        // than exists) has no safe window. Blocks narrower than the column
        // pitch cannot cause it, but off-grid blocks can. Keep the lanes
        // distinct around the middle of the violation.
        let w = (gi.safe_max - gi.safe_min).max(0.0);
        let center = (gi.safe_min + gi.safe_max) / 2.0;
        let step = if w > 0.0 {
            p.lane_step.min(w / total.max(1) as f64)
        } else {
            1.0
        };
        return center + offset(step);
    }
    if bounded_right {
        return gi.safe_max - slot as f64 * p.lane_step;
    }
    if bounded_left {
        return gi.safe_min + slot as f64 * p.lane_step;
    }
    natural_x + offset(p.lane_step)
}

/// The vertical run of one net in one gutter.
#[derive(Clone, Copy)]
struct Span {
    net: usize,
    lo: f64,
    hi: f64,
}

/// Assign each (net, gutter) pair a lane X.
///
/// Nets whose vertical runs in a gutter are clear of each other share one
/// lane (interval packing, first fit by top edge). A gutter then needs only
/// as many lanes as its busiest height, so the lanes that remain keep their
/// full spacing. Runs on one lane keep a gap of two lane steps, so they
/// never read as one wire.
///
/// Lanes are then ordered by the mean vertical midpoint of their nets, so
/// nets flowing in parallel keep their relative order instead of weaving.
fn assign_lanes(
    per_gutter: &HashMap<i32, Vec<Span>>,
    info: &HashMap<i32, GutterInfo>,
    p: Params,
) -> HashMap<(usize, i32), f64> {
    let share_gap = 2.0 * p.lane_step;
    let mut lanes: HashMap<(usize, i32), f64> = HashMap::new();
    for (&idx, spans) in per_gutter {
        let mut ordered = spans.clone();
        ordered.sort_by(|a, b| a.lo.partial_cmp(&b.lo).unwrap().then(a.net.cmp(&b.net)));

        // (bottom of the last run, member nets, sum of member midpoints)
        let mut tracks: Vec<(f64, Vec<usize>, f64)> = Vec::new();
        for s in ordered {
            let mid = (s.lo + s.hi) / 2.0;
            match tracks.iter_mut().find(|t| t.0 + share_gap <= s.lo) {
                Some(t) => {
                    t.0 = s.hi;
                    t.1.push(s.net);
                    t.2 += mid;
                }
                None => tracks.push((s.hi, vec![s.net], mid)),
            }
        }
        tracks.sort_by(|a, b| {
            let (ma, mb) = (a.2 / a.1.len() as f64, b.2 / b.1.len() as f64);
            ma.partial_cmp(&mb).unwrap().then(a.1[0].cmp(&b.1[0]))
        });
        let total = tracks.len();
        for (slot, track) in tracks.iter().enumerate() {
            let x = lane_x(idx, slot, total, info, p);
            for &net in &track.1 {
                lanes.insert((net, idx), x);
            }
        }
    }
    lanes
}

/// The Y nearest to `preferred` that lies in none of the `blocked` closed
/// intervals.
fn nearest_free_y(preferred: f64, blocked: &[(f64, f64)]) -> f64 {
    let is_blocked = |y: f64| blocked.iter().any(|b| y >= b.0 && y <= b.1);
    if !is_blocked(preferred) {
        return preferred;
    }
    let mut candidates: Vec<f64> = Vec::new();
    for b in blocked {
        candidates.push(b.0 - 1.0);
        candidates.push(b.1 + 1.0);
    }
    candidates.sort_by(|a, b| {
        (a - preferred)
            .abs()
            .partial_cmp(&(b - preferred).abs())
            .unwrap()
    });
    // The candidate above the highest interval is always free.
    candidates
        .into_iter()
        .find(|&y| !is_blocked(y))
        .unwrap_or(preferred)
}

/// Route every net. `nets` must be in a stable order (the caller sorts by
/// net key): bridge placement and lane ties break on input order.
pub fn plan_routes(nets: &[Net], obstacles: &[Rect], p: Params) -> RouteResult {
    const BODY_MARGIN: f64 = 8.0;
    let info = build_gutter_info(nets, obstacles, p);
    let mut result = RouteResult::default();

    struct NetPlan {
        g_d: i32,
        g_l: Vec<i32>,
        unique_g: Vec<i32>, // sorted
        hy: Option<f64>,    // bridge Y (multi-gutter nets only)
    }

    // Phase 1: the gutters each net uses.
    let mut plans: Vec<NetPlan> = nets
        .iter()
        .map(|net| {
            let g_d = gutter_index(net.driver.col, net.driver.exits_right);
            let g_l: Vec<i32> = net
                .loads
                .iter()
                .map(|l| gutter_index(l.col, l.exits_right))
                .collect();
            let mut unique_g: Vec<i32> = g_l.clone();
            unique_g.push(g_d);
            unique_g.sort_unstable();
            unique_g.dedup();
            NetPlan {
                g_d,
                g_l,
                unique_g,
                hy: None,
            }
        })
        .collect();

    // Phase 2: bridge Y for every net that spans gutters. The preferred Y
    // is the mean of the pin Ys. The bridge moves to the nearest Y that is
    // clear of:
    //
    // * block bodies in the columns it crosses;
    // * the pin stubs of other nets in the gutters it crosses (a bridge on
    //   a stub reads as one wire);
    // * bridges placed before it that cross a common gutter.
    //
    // This runs before lane assignment, on gutter indices and not on lane
    // X, so that lane assignment sees the final vertical extent of each net.
    let near = p.lane_step - 1.0; // closed band half-width; next free Y is one lane step away
    let mut bridges: Vec<(i32, i32, f64)> = Vec::new(); // (first gutter, last gutter, Y)
    for (net_idx, net) in nets.iter().enumerate() {
        let (g_min, g_max) = {
            let u = &plans[net_idx].unique_g;
            (u[0], u[u.len() - 1])
        };
        if g_min == g_max {
            continue;
        }
        let bx_min = gutter_center_x(g_min, p.column_pitch);
        let bx_max = gutter_center_x(g_max, p.column_pitch);
        let mut blocked: Vec<(f64, f64)> = obstacles
            .iter()
            .filter(|r| !(r.right < bx_min || r.left > bx_max))
            .map(|r| (r.top - BODY_MARGIN, r.bottom + BODY_MARGIN))
            .collect();
        for (other_idx, other) in nets.iter().enumerate() {
            if other_idx == net_idx {
                continue;
            }
            for e in std::iter::once(&other.driver).chain(&other.loads) {
                let g = gutter_index(e.col, e.exits_right);
                if g >= g_min && g <= g_max {
                    blocked.push((e.y - near, e.y + near));
                }
            }
        }
        for &(b_min, b_max, y) in &bridges {
            if b_min <= g_max && b_max >= g_min {
                blocked.push((y - near, y + near));
            }
        }
        let sum_y: f64 = net.driver.y + net.loads.iter().map(|l| l.y).sum::<f64>();
        let hy = nearest_free_y(sum_y / (1 + net.loads.len()) as f64, &blocked);
        bridges.push((g_min, g_max, hy));
        plans[net_idx].hy = Some(hy);
    }

    // Phase 3: lanes. The vertical run of a net in a gutter covers its pins
    // there and, for a bridged net, the bridge Y.
    let mut per_gutter: HashMap<i32, Vec<Span>> = HashMap::new();
    for (net_idx, net) in nets.iter().enumerate() {
        let plan = &plans[net_idx];
        for &g in &plan.unique_g {
            let mut ys: Vec<f64> = Vec::new();
            if plan.g_d == g {
                ys.push(net.driver.y);
            }
            for (i, l) in net.loads.iter().enumerate() {
                if plan.g_l[i] == g {
                    ys.push(l.y);
                }
            }
            if let Some(hy) = plan.hy {
                ys.push(hy);
            }
            per_gutter.entry(g).or_default().push(Span {
                net: net_idx,
                lo: ys.iter().cloned().fold(f64::INFINITY, f64::min),
                hi: ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            });
        }
    }
    let lanes = assign_lanes(&per_gutter, &info, p);

    // Phase 4: emit polylines and junction dots.
    for (net_idx, net) in nets.iter().enumerate() {
        let plan = &plans[net_idx];
        let driver = net.driver;
        let g_d = plan.g_d;
        let g_l = &plan.g_l;

        let mut dot_points: Vec<Point> = Vec::new();

        if let Some(hy) = plan.hy {
            // Bridge route: stub, drop to the bridge, bridge, rise, stub.
            let mut gx_for_idx: HashMap<i32, f64> = HashMap::new();
            for &idx in &plan.unique_g {
                gx_for_idx.insert(idx, lanes[&(net_idx, idx)]);
            }
            let dgx = gx_for_idx[&g_d];

            for (i, l) in net.loads.iter().enumerate() {
                let lgx = gx_for_idx[&g_l[i]];
                let mut wp = vec![
                    Point {
                        x: driver.x,
                        y: driver.y,
                    },
                    Point {
                        x: dgx,
                        y: driver.y,
                    },
                    Point { x: dgx, y: hy },
                ];
                if lgx != dgx {
                    wp.push(Point { x: lgx, y: hy });
                }
                wp.push(Point { x: lgx, y: l.y });
                wp.push(Point { x: l.x, y: l.y });
                result.wires.push(wp);
            }

            // Dots at interior taps along the bridge…
            let mut tap_xs: Vec<f64> = gx_for_idx.values().copied().collect();
            tap_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            tap_xs.dedup();
            if tap_xs.len() > 2 {
                for &x in &tap_xs[1..tap_xs.len() - 1] {
                    dot_points.push(Point { x, y: hy });
                }
            }
            // …and at interior fan-out points within each shared gutter.
            let mut ys_at_gutter: HashMap<i32, Vec<f64>> = HashMap::new();
            ys_at_gutter.entry(g_d).or_default().push(driver.y);
            for (i, l) in net.loads.iter().enumerate() {
                ys_at_gutter.entry(g_l[i]).or_default().push(l.y);
            }
            for (idx, ys) in &ys_at_gutter {
                if ys.len() < 2 {
                    continue;
                }
                let mut sorted = ys.clone();
                sorted.push(hy);
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let gx = gx_for_idx[idx];
                for &y in &sorted[1..sorted.len() - 1] {
                    if (y - hy).abs() < 0.5 {
                        continue;
                    }
                    dot_points.push(Point { x: gx, y });
                }
            }
        } else {
            // Everything shares one gutter: stub → vertical trunk → stub.
            let gx = lanes[&(net_idx, g_d)];
            for l in &net.loads {
                result.wires.push(vec![
                    Point {
                        x: driver.x,
                        y: driver.y,
                    },
                    Point { x: gx, y: driver.y },
                    Point { x: gx, y: l.y },
                    Point { x: l.x, y: l.y },
                ]);
            }
            if net.loads.len() >= 2 {
                let mut ys: Vec<f64> = vec![driver.y];
                ys.extend(net.loads.iter().map(|l| l.y));
                ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
                for &y in &ys[1..ys.len() - 1] {
                    dot_points.push(Point { x: gx, y });
                }
            }
        }

        // Dedupe per net on rounded coordinates.
        let mut seen: HashSet<(i64, i64)> = HashSet::new();
        for pt in dot_points {
            let key = (pt.x.round() as i64, pt.y.round() as i64);
            if seen.insert(key) {
                result.dots.push(Dot {
                    x: pt.x,
                    y: pt.y,
                    net: net_idx,
                });
            }
        }
    }
    result
}

/// One module in a column, for push-and-shove legalization.
#[derive(Clone, Copy, Debug)]
pub struct ColItem {
    pub id: i32,
    pub col: i32,
    pub top: f64,
    pub height: f64,
    /// Fixed items (the dragged selection) keep their Y; everything else
    /// shifts minimally.
    pub fixed: bool,
}

/// Push-and-shove placement: per column, keep every `fixed` item exactly
/// where it is and shift the others the minimum distance that restores at
/// least `gap` of separation, preserving vertical order. Returns
/// `(id, new_top)` for the items that actually moved.
pub fn legalize_columns(items: &[ColItem], gap: f64) -> Vec<(i32, f64)> {
    let mut cols: HashMap<i32, Vec<ColItem>> = HashMap::new();
    for it in items {
        cols.entry(it.col).or_default().push(*it);
    }
    let mut moves: Vec<(i32, f64)> = Vec::new();
    for (_, mut col) in cols {
        // Original vertical order; fixed first on ties so equal-top movables
        // yield to the dragged block.
        col.sort_by(|a, b| {
            a.top
                .partial_cmp(&b.top)
                .unwrap()
                .then(b.fixed.cmp(&a.fixed))
        });
        let mut assigned: Vec<f64> = col.iter().map(|c| c.top).collect();
        let mut cursor = f64::NEG_INFINITY;
        // Movables placed since the last fixed item — the run that shifts up
        // when a fixed item's position overrides the cursor.
        let mut run: Vec<usize> = Vec::new();
        for i in 0..col.len() {
            if col[i].fixed {
                let ftop = col[i].top;
                if cursor > ftop {
                    let delta = cursor - ftop;
                    for &j in &run {
                        assigned[j] -= delta;
                    }
                }
                cursor = ftop + col[i].height + gap;
                run.clear();
            } else {
                let t = col[i].top.max(cursor);
                assigned[i] = t;
                cursor = t + col[i].height + gap;
                run.push(i);
            }
        }
        for (i, it) in col.iter().enumerate() {
            if !it.fixed && (assigned[i] - it.top).abs() > 0.5 {
                moves.push((it.id, assigned[i]));
            }
        }
    }
    moves.sort_by_key(|m| m.0);
    moves
}

// --- Layout optimizer -------------------------------------------------------

/// One block to place. `height` includes the pin rows.
#[derive(Clone, Copy, Debug)]
pub struct PlaceNode {
    pub id: i32,
    pub height: f64,
}

/// A wire between two pins; `*_dy` is the pin's Y offset from its block's
/// top edge, so straightening can align pins, not blocks. `directed` means
/// `from` electrically drives `to`; undirected edges (load-to-load nets)
/// pull blocks together but never force a column ordering.
#[derive(Clone, Copy, Debug)]
pub struct PlaceEdge {
    pub from: i32,
    pub from_dy: f64,
    pub to: i32,
    pub to_dy: f64,
    pub directed: bool,
}

/// Optimized placement: (id, column, top Y).
///
/// 1. Columns by longest path over the directed edges (drivers left of
///    loads). Cycle back-edges are dropped via DFS first, so feedback
///    loops can't inflate the column count; columns compress dense, and
///    `max_cols` (when > 0) hard-caps the total — Tidy never spreads the
///    design wider than it already is.
/// 2. Barycenter sweeps pull each block toward the mean of its connected
///    pins, legalizing every column (order kept, gap restored) per sweep.
/// 3. A final left→right straightening pass snaps each block so its
///    input pin aligns exactly with the driving pin — straight wires
///    wherever the column packing allows.
pub fn optimize_positions(
    nodes: &[PlaceNode],
    edges: &[PlaceEdge],
    gap: f64,
    max_cols: usize,
) -> Vec<(i32, i32, f64)> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let index: HashMap<i32, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let n = nodes.len();
    let edges: Vec<(usize, f64, usize, f64, bool)> = edges
        .iter()
        .filter_map(|e| {
            let (f, t) = (*index.get(&e.from)?, *index.get(&e.to)?);
            if f == t {
                return None;
            }
            Some((f, e.from_dy, t, e.to_dy, e.directed))
        })
        .collect();

    // --- 1a. Drop cycle back-edges (iterative DFS, three-color) ---
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n]; // edge indices, directed only
    for (ei, &(f, _, _, _, directed)) in edges.iter().enumerate() {
        if directed {
            adj[f].push(ei);
        }
    }
    let mut color = vec![0u8; n]; // 0 white, 1 on-stack, 2 done
    let mut keep = vec![true; edges.len()];
    for start in 0..n {
        if color[start] != 0 {
            continue;
        }
        // Stack of (node, next child index to visit).
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        color[start] = 1;
        while let Some(&mut (u, ref mut next)) = stack.last_mut() {
            if *next < adj[u].len() {
                let ei = adj[u][*next];
                *next += 1;
                let v = edges[ei].2;
                match color[v] {
                    0 => {
                        color[v] = 1;
                        stack.push((v, 0));
                    }
                    1 => keep[ei] = false, // back edge: closes a cycle
                    _ => {}
                }
            } else {
                color[u] = 2;
                stack.pop();
            }
        }
    }

    // --- 1b. Longest-path layering over the remaining DAG ---
    let mut col: Vec<i32> = vec![0; n];
    for _ in 0..n {
        let mut changed = false;
        for (ei, &(f, _, t, _, directed)) in edges.iter().enumerate() {
            if directed && keep[ei] && col[t] < col[f] + 1 {
                col[t] = col[f] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // --- 1c. Compress to dense columns, capped at max_cols ---
    let mut used: Vec<i32> = col.clone();
    used.sort_unstable();
    used.dedup();
    let rank: HashMap<i32, i32> = used
        .iter()
        .enumerate()
        .map(|(r, &c)| (c, r as i32))
        .collect();
    let precap: Vec<i32> = col.iter().map(|c| rank[c]).collect();
    for c in col.iter_mut() {
        *c = rank[c];
        if max_cols > 0 {
            *c = (*c).min(max_cols as i32 - 1);
        }
    }

    // --- 1d. Spread: use the whole column budget. Cramming independent
    // blocks into one column funnels every input through one gutter; while
    // columns remain unused, split the tallest column in two (later ranks
    // and lower blocks move right, so edge direction survives).
    if max_cols > 0 {
        loop {
            let mut by_col: HashMap<i32, Vec<usize>> = HashMap::new();
            for (i, &c) in col.iter().enumerate() {
                by_col.entry(c).or_default().push(i);
            }
            if by_col.len() >= max_cols {
                break;
            }
            let tallest = by_col
                .iter()
                .filter(|(_, m)| m.len() >= 2)
                .max_by(|a, b| {
                    let h = |m: &Vec<usize>| m.iter().map(|&i| nodes[i].height + gap).sum::<f64>();
                    h(a.1).partial_cmp(&h(b.1)).unwrap()
                })
                .map(|(&c, m)| (c, m.clone()));
            let Some((tc, mut members)) = tallest else {
                break;
            };
            members.sort_by(|&a, &b| precap[a].cmp(&precap[b]).then(a.cmp(&b)));
            for c in col.iter_mut() {
                if *c > tc {
                    *c += 1;
                }
            }
            let half = members.len() / 2;
            for &i in &members[half..] {
                col[i] = tc + 1;
            }
        }
    }

    // --- 2. Barycenter sweeps with per-column legalization ---
    // Initial stacking: per column, in input order.
    let mut y: Vec<f64> = vec![0.0; n];
    let mut by_col: HashMap<i32, Vec<usize>> = HashMap::new();
    for (i, &c) in col.iter().enumerate() {
        by_col.entry(c).or_default().push(i);
    }
    for members in by_col.values() {
        let mut cur = 0.0;
        for &i in members {
            y[i] = cur;
            cur += nodes[i].height + gap;
        }
    }
    let legalize = |y: &mut Vec<f64>, order: &mut Vec<usize>, members: &[usize]| {
        // Stack in desired-Y order around the group's mean so columns don't
        // drift; restores the minimum gap.
        let mut m: Vec<usize> = members.to_vec();
        m.sort_by(|&a, &b| y[a].partial_cmp(&y[b]).unwrap().then(a.cmp(&b)));
        let total: f64 = m.iter().map(|&i| nodes[i].height).sum::<f64>()
            + gap * (m.len().saturating_sub(1)) as f64;
        let mean: f64 =
            m.iter().map(|&i| y[i] + nodes[i].height / 2.0).sum::<f64>() / m.len() as f64;
        let mut cur = mean - total / 2.0;
        for &i in &m {
            y[i] = cur;
            cur += nodes[i].height + gap;
        }
        *order = m;
    };
    let mut orders: HashMap<i32, Vec<usize>> = by_col.clone();
    for _ in 0..8 {
        for i in 0..n {
            let mut sum = 0.0;
            let mut cnt = 0.0;
            for &(f, fdy, t, tdy, _) in &edges {
                if t == i {
                    sum += y[f] + fdy - tdy;
                    cnt += 1.0;
                } else if f == i {
                    sum += y[t] + tdy - fdy;
                    cnt += 1.0;
                }
            }
            if cnt > 0.0 {
                y[i] = sum / cnt;
            }
        }
        for (c, members) in &by_col {
            let mut order = Vec::new();
            legalize(&mut y, &mut order, members);
            orders.insert(*c, order);
        }
    }

    // --- 3a. Driver centering (right→left): a fan-out driver sits with its
    // output pin at the mean of the input pins it feeds, so the fan spreads
    // symmetrically instead of bundling behind the loads.
    let mut cols_sorted: Vec<i32> = by_col.keys().copied().collect();
    cols_sorted.sort_unstable();
    for &c in cols_sorted.iter().rev() {
        let members = orders.get(&c).cloned().unwrap_or_default();
        for &i in &members {
            let targets: Vec<f64> = edges
                .iter()
                .filter(|&&(f, _, t, _, _)| f == i && col[t] > c)
                .map(|&(_, fdy, t, tdy, _)| y[t] + tdy - fdy)
                .collect();
            if targets.len() >= 2 {
                y[i] = targets.iter().sum::<f64>() / targets.len() as f64;
            }
        }
        let mut order = Vec::new();
        legalize(&mut y, &mut order, &members);
        orders.insert(c, order);
    }

    // --- 3b. Straightening: align each block's first input pin exactly ---
    for &c in &cols_sorted[1..] {
        let members = orders.get(&c).cloned().unwrap_or_default();
        for &i in &members {
            if let Some(&(f, fdy, _, tdy, _)) =
                edges.iter().find(|&&(f, _, t, _, _)| t == i && col[f] < c)
            {
                y[i] = y[f] + fdy - tdy;
            }
        }
        // Re-legalize in straightened order; ties keep barycenter order.
        let mut order = Vec::new();
        legalize(&mut y, &mut order, &members);
        orders.insert(c, order);
    }

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, col[i], y[i]))
        .collect()
}

/// Find a Y where a `width`×`height` module at `left` doesn't overlap any
/// other module (each side padded by `margin`). Prefers `proposed_y`, else
/// the nearest clear candidate, else below everything.
pub fn resolve_clear_y(
    left: f64,
    width: f64,
    height: f64,
    proposed_y: f64,
    margin: f64,
    obstacles: &[Rect],
) -> f64 {
    let probe_left = left - margin;
    let probe_right = left + width + margin;
    let blocking: Vec<Rect> = obstacles
        .iter()
        .map(|r| Rect {
            left: r.left - margin,
            top: r.top - margin,
            right: r.right + margin,
            bottom: r.bottom + margin,
        })
        .filter(|r| !(r.right < probe_left || r.left > probe_right))
        .collect();
    if blocking.is_empty() {
        return proposed_y;
    }

    let clear_at = |y: f64| -> bool {
        let top = y - margin;
        let bottom = y + height + margin;
        !blocking.iter().any(|b| {
            b.left < probe_right && b.right > probe_left && b.top < bottom && b.bottom > top
        })
    };

    if clear_at(proposed_y) {
        return proposed_y;
    }

    let mut candidates: Vec<f64> = Vec::new();
    for b in &blocking {
        candidates.push(b.top - height);
        candidates.push(b.bottom);
    }
    candidates.sort_by(|a, b| {
        (a - proposed_y)
            .abs()
            .partial_cmp(&(b - proposed_y).abs())
            .unwrap()
    });
    for y in candidates {
        if clear_at(y) {
            return y;
        }
    }
    blocking.iter().map(|b| b.bottom).fold(proposed_y, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Params = Params {
        column_pitch: 480.0,
        lane_step: 12.0,
        stub_min: 40.0,
    };

    fn ep(x: f64, y: f64, exits_right: bool, col: i32) -> Endpoint {
        Endpoint {
            x,
            y,
            exits_right,
            col,
        }
    }

    #[test]
    fn same_gutter_is_four_points() {
        // Driver exits right from col 0, load enters from the left of col 1:
        // both use gutter index 1.
        let nets = [Net {
            driver: ep(180.0, 100.0, true, 0),
            loads: vec![ep(300.0, 300.0, false, 1)],
        }];
        let r = plan_routes(&nets, &[], P);
        assert_eq!(r.wires.len(), 1);
        let w = &r.wires[0];
        assert_eq!(w.len(), 4);
        assert_eq!((w[0].x, w[0].y), (180.0, 100.0));
        assert_eq!((w[3].x, w[3].y), (300.0, 300.0));
        // Trunk is vertical: middle points share X.
        assert_eq!(w[1].x, w[2].x);
        // Stub minimums respected on both sides.
        assert!(w[1].x >= 180.0 + P.stub_min);
        assert!(w[1].x <= 300.0 - P.stub_min);
        assert!(r.dots.is_empty());
    }

    #[test]
    fn fan_out_gets_interior_dots() {
        let nets = [Net {
            driver: ep(180.0, 100.0, true, 0),
            loads: vec![
                ep(300.0, 200.0, false, 1),
                ep(300.0, 300.0, false, 1),
                ep(300.0, 400.0, false, 1),
            ],
        }];
        let r = plan_routes(&nets, &[], P);
        assert_eq!(r.wires.len(), 3);
        // Interior taps: 4 pin Ys sorted → 2 interior.
        assert_eq!(r.dots.len(), 2);
        assert!(r.dots.iter().all(|d| d.net == 0));
    }

    #[test]
    fn overlapping_runs_in_one_gutter_get_distinct_lanes() {
        // Vertical runs 100..200 and 150..250 overlap.
        let nets = [
            Net {
                driver: ep(180.0, 100.0, true, 0),
                loads: vec![ep(300.0, 200.0, false, 1)],
            },
            Net {
                driver: ep(180.0, 150.0, true, 0),
                loads: vec![ep(300.0, 250.0, false, 1)],
            },
        ];
        let r = plan_routes(&nets, &[], P);
        assert!(
            (r.wires[0][1].x - r.wires[1][1].x).abs() >= P.lane_step,
            "lanes must be a full step apart"
        );
    }

    #[test]
    fn clear_runs_in_one_gutter_share_a_lane() {
        // Runs 100..150 and 200..250 never meet: one lane serves both, so
        // the gutter keeps its other lanes free.
        let nets = [
            Net {
                driver: ep(180.0, 100.0, true, 0),
                loads: vec![ep(300.0, 150.0, false, 1)],
            },
            Net {
                driver: ep(180.0, 200.0, true, 0),
                loads: vec![ep(300.0, 250.0, false, 1)],
            },
        ];
        let r = plan_routes(&nets, &[], P);
        assert_eq!(r.wires[0][1].x, r.wires[1][1].x);
    }

    fn segments_intersect(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
        // Proper crossing of one horizontal and one vertical segment.
        let (h, v) = if (a0.y - a1.y).abs() < 1e-9 && (b0.x - b1.x).abs() < 1e-9 {
            ((a0, a1), (b0, b1))
        } else if (b0.y - b1.y).abs() < 1e-9 && (a0.x - a1.x).abs() < 1e-9 {
            ((b0, b1), (a0, a1))
        } else {
            return false;
        };
        let (hx0, hx1) = (h.0.x.min(h.1.x), h.0.x.max(h.1.x));
        let (vy0, vy1) = (v.0.y.min(v.1.y), v.0.y.max(v.1.y));
        v.0.x > hx0 && v.0.x < hx1 && h.0.y > vy0 && h.0.y < vy1
    }

    fn crossings(a: &[Point], b: &[Point]) -> usize {
        let mut n = 0;
        for sa in a.windows(2) {
            for sb in b.windows(2) {
                if segments_intersect(sa[0], sa[1], sb[0], sb[1]) {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn parallel_nets_in_one_gutter_do_not_cross() {
        // Two nets flowing top→top and bottom→bottom through one gutter.
        // Order-blind lane allocation used to weave these.
        let nets = [
            Net {
                driver: ep(180.0, 100.0, true, 0),
                loads: vec![ep(300.0, 120.0, false, 1)],
            },
            Net {
                driver: ep(180.0, 200.0, true, 0),
                loads: vec![ep(300.0, 220.0, false, 1)],
            },
            Net {
                driver: ep(180.0, 300.0, true, 0),
                loads: vec![ep(300.0, 320.0, false, 1)],
            },
        ];
        let r = plan_routes(&nets, &[], P);
        for i in 0..r.wires.len() {
            for j in i + 1..r.wires.len() {
                assert_eq!(
                    crossings(&r.wires[i], &r.wires[j]),
                    0,
                    "wires {i} and {j} cross"
                );
            }
        }
    }

    #[test]
    fn overlapping_bridges_get_separated() {
        // Two long bridges with identical mean Y and overlapping X span
        // must not collapse onto one horizontal line.
        let nets = [
            Net {
                driver: ep(20.0, 100.0, false, 0),
                loads: vec![ep(1660.0, 300.0, true, 3)],
            },
            Net {
                driver: ep(20.0, 150.0, false, 0),
                loads: vec![ep(1660.0, 250.0, true, 3)],
            },
        ];
        let r = plan_routes(&nets, &[], P);
        let hy0 = r.wires[0][2].y;
        let hy1 = r.wires[1][2].y;
        assert!(
            (hy0 - hy1).abs() >= P.lane_step,
            "bridges share a band: {hy0} vs {hy1}"
        );
    }

    #[test]
    fn bridge_shifts_off_module_body() {
        // Driver in col 0 exiting LEFT, load in col 1 entering from the
        // RIGHT: gutters -1 and 3, so the bridge spans the whole middle —
        // including a module parked at the mean Y.
        let nets = [Net {
            driver: ep(20.0, 100.0, false, 0),
            loads: vec![ep(700.0, 300.0, true, 1)],
        }];
        let block = Rect {
            left: 100.0,
            top: 150.0,
            right: 600.0,
            bottom: 250.0,
        };
        let r = plan_routes(&nets, &[block], P);
        let hy = r.wires[0][2].y; // bridge Y
        assert!(
            !(hy >= block.top - 8.0 && hy <= block.bottom + 8.0),
            "bridge y {hy} sits inside blocked band"
        );
    }

    // --- Property tests: random designs on the column grid -----------------

    use proptest::prelude::*;

    // The canvas leaves a gutter of 200 between two blocks of maximum
    // width (2 * kColumnGutterHalf). Same gutter here, on the test pitch.
    const MAX_BLOCK_WIDTH: f64 = 280.0;
    const PIN_SLOT: f64 = 24.0; // kPinSlotHeight

    /// A random design: blocks centered on their column, pins on the block
    /// edges, and nets that each own their pins.
    fn design() -> impl Strategy<Value = (Vec<Net>, Vec<Rect>)> {
        // Per column: up to 3 blocks as (half width, pin rows, gap above).
        let column = prop::collection::vec(
            (100.0..=MAX_BLOCK_WIDTH / 2.0, 2usize..8, 60.0..400.0f64),
            0..3,
        );
        (prop::collection::vec(column, 2..5), any::<u64>()).prop_map(|(cols, seed)| {
            let mut rects = Vec::new();
            let mut outs: Vec<Endpoint> = Vec::new(); // right-edge pins
            let mut ins: Vec<Endpoint> = Vec::new(); // left-edge pins
            for (c, blocks) in cols.iter().enumerate() {
                let cx = c as f64 * P.column_pitch;
                let mut y = 0.0;
                for &(half, rows, gap) in blocks {
                    y += gap;
                    let top = y;
                    let bottom = top + 50.0 + rows as f64 * PIN_SLOT;
                    rects.push(Rect {
                        left: cx - half,
                        top,
                        right: cx + half,
                        bottom,
                    });
                    for r in 0..rows {
                        let py = top + 50.0 + (r as f64 + 0.5) * PIN_SLOT;
                        outs.push(ep(cx + half, py, true, c as i32));
                        ins.push(ep(cx - half, py, false, c as i32));
                    }
                    y = bottom;
                }
            }
            // Deterministic pseudo-random pairing from the seed: each net
            // takes one output pin and one to three input pins.
            let mut state = seed | 1;
            let mut next = |n: usize| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state % n.max(1) as u64) as usize
            };
            let mut nets = Vec::new();
            while !outs.is_empty() && !ins.is_empty() && nets.len() < 24 {
                let driver = outs.swap_remove(next(outs.len()));
                let mut loads = Vec::new();
                for _ in 0..1 + next(3) {
                    if ins.is_empty() {
                        break;
                    }
                    loads.push(ins.swap_remove(next(ins.len())));
                }
                nets.push(Net { driver, loads });
            }
            (nets, rects)
        })
    }

    /// (net index, segment start, segment end, is a pin stub).
    fn segments(nets: &[Net], r: &RouteResult) -> Vec<(usize, Point, Point, bool)> {
        let mut out = Vec::new();
        let mut wi = 0;
        for (ni, net) in nets.iter().enumerate() {
            for _ in &net.loads {
                let w = &r.wires[wi];
                wi += 1;
                for (i, s) in w.windows(2).enumerate() {
                    out.push((ni, s[0], s[1], i == 0 || i == w.len() - 2));
                }
            }
        }
        out
    }

    fn is_vertical(a: Point, b: Point) -> bool {
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() > 1e-9
    }

    fn is_horizontal(a: Point, b: Point) -> bool {
        (a.y - b.y).abs() < 1e-9 && (a.x - b.x).abs() > 1e-9
    }

    proptest! {
        /// A wire never runs through the body of a block.
        #[test]
        fn wires_stay_out_of_block_bodies((nets, rects) in design()) {
            let r = plan_routes(&nets, &rects, P);
            for (_, a, b, _) in segments(&nets, &r) {
                for rect in &rects {
                    // Sample the segment; 1 px inside the edge counts as clear.
                    for k in 0..=64 {
                        let t = k as f64 / 64.0;
                        let (x, y) = (a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
                        prop_assert!(
                            !rect.contains_strict(x, y, 1.0),
                            "segment {a:?}-{b:?} enters block {rect:?}"
                        );
                    }
                }
            }
        }

        /// Two nets never draw one vertical run on top of the other, and
        /// runs that share a lane keep a visible gap between them.
        #[test]
        fn vertical_runs_of_different_nets_stay_apart((nets, rects) in design()) {
            let r = plan_routes(&nets, &rects, P);
            let segs = segments(&nets, &r);
            let vert: Vec<_> = segs.iter().filter(|s| is_vertical(s.1, s.2)).collect();
            for a in &vert {
                for b in &vert {
                    if a.0 >= b.0 || (a.1.x - b.1.x).abs() > 1e-6 {
                        continue;
                    }
                    let (a0, a1) = (a.1.y.min(a.2.y), a.1.y.max(a.2.y));
                    let (b0, b1) = (b.1.y.min(b.2.y), b.1.y.max(b.2.y));
                    prop_assert!(
                        a1 + P.lane_step <= b0 || b1 + P.lane_step <= a0,
                        "nets {} and {} share lane x={} with runs {a0}..{a1} and {b0}..{b1}",
                        a.0, b.0, a.1.x
                    );
                }
            }
        }

        /// A bridge never lies on another net's bridge or pin stub.
        #[test]
        fn bridges_do_not_cover_other_nets((nets, rects) in design()) {
            let r = plan_routes(&nets, &rects, P);
            let segs = segments(&nets, &r);
            let horiz: Vec<_> = segs.iter().filter(|s| is_horizontal(s.1, s.2)).collect();
            for bridge in horiz.iter().filter(|s| !s.3) {
                for other in &horiz {
                    if other.0 == bridge.0 || (other.1.y - bridge.1.y).abs() >= P.lane_step - 1.0 {
                        continue;
                    }
                    let (a0, a1) = (bridge.1.x.min(bridge.2.x), bridge.1.x.max(bridge.2.x));
                    let (b0, b1) = (other.1.x.min(other.2.x), other.1.x.max(other.2.x));
                    prop_assert!(
                        a1 <= b0 || b1 <= a0,
                        "bridge of net {} at y={} covers net {} at y={} (x {a0}..{a1} vs {b0}..{b1})",
                        bridge.0, bridge.1.y, other.0, other.1.y
                    );
                }
            }
        }
    }

    fn ci(id: i32, top: f64, height: f64, fixed: bool) -> ColItem {
        ColItem {
            id,
            col: 0,
            top,
            height,
            fixed,
        }
    }

    #[test]
    fn legalize_noop_when_clear() {
        let items = [ci(0, 0.0, 100.0, true), ci(1, 200.0, 100.0, false)];
        assert!(legalize_columns(&items, 60.0).is_empty());
    }

    #[test]
    fn legalize_pushes_lower_neighbor_down_and_cascades() {
        // Fixed dropped onto item 1; item 1 pushes down into item 2.
        let items = [
            ci(0, 100.0, 100.0, true),
            ci(1, 150.0, 100.0, false),
            ci(2, 320.0, 100.0, false),
        ];
        let moves = legalize_columns(&items, 60.0);
        assert_eq!(moves, vec![(1, 260.0), (2, 420.0)]);
    }

    #[test]
    fn legalize_shifts_upper_run_up() {
        // Fixed lands overlapping an item that sits slightly above it — the
        // upper item yields upward, exactly clearing the gap.
        let items = [ci(0, 90.0, 100.0, false), ci(1, 100.0, 100.0, true)];
        let moves = legalize_columns(&items, 60.0);
        assert_eq!(moves, vec![(0, -60.0)]);
    }

    #[test]
    fn legalize_never_moves_fixed_and_ignores_other_columns() {
        let mut other = ci(3, 100.0, 100.0, false);
        other.col = 1; // different column: untouched even though Y overlaps
        let items = [ci(0, 100.0, 100.0, true), ci(1, 100.0, 100.0, false), other];
        let moves = legalize_columns(&items, 60.0);
        assert_eq!(moves, vec![(1, 260.0)]);
    }

    fn pn(id: i32, height: f64) -> PlaceNode {
        PlaceNode { id, height }
    }
    fn pe(from: i32, from_dy: f64, to: i32, to_dy: f64) -> PlaceEdge {
        PlaceEdge {
            from,
            from_dy,
            to,
            to_dy,
            directed: true,
        }
    }

    #[test]
    fn optimize_layers_follow_signal_flow() {
        // a -> b -> c chain: columns 0, 1, 2.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 100.0)];
        let edges = [pe(0, 50.0, 1, 50.0), pe(1, 50.0, 2, 50.0)];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        let col_of = |id: i32| r.iter().find(|e| e.0 == id).unwrap().1;
        assert_eq!((col_of(0), col_of(1), col_of(2)), (0, 1, 2));
    }

    #[test]
    fn optimize_straightens_chain() {
        // Matching pin offsets → connected blocks align exactly.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 100.0)];
        let edges = [pe(0, 30.0, 1, 70.0), pe(1, 30.0, 2, 30.0)];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        let y_of = |id: i32| r.iter().find(|e| e.0 == id).unwrap().2;
        // b's input pin (dy 70) aligns with a's output pin (dy 30).
        assert!((y_of(0) + 30.0 - (y_of(1) + 70.0)).abs() < 0.5, "{r:?}");
        // c's input aligns with b's output (same dy → same top).
        assert!((y_of(1) - y_of(2)).abs() < 0.5, "{r:?}");
    }

    #[test]
    fn optimize_fanout_stacks_with_gap() {
        // One driver, three loads in the same column: loads stacked, gapped.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 100.0), pn(3, 100.0)];
        let edges = [
            pe(0, 50.0, 1, 50.0),
            pe(0, 50.0, 2, 50.0),
            pe(0, 50.0, 3, 50.0),
        ];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        let mut load_ys: Vec<f64> = r.iter().filter(|e| e.0 != 0).map(|e| e.2).collect();
        load_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((load_ys[1] - load_ys[0] - 160.0).abs() < 0.5, "{load_ys:?}");
        assert!((load_ys[2] - load_ys[1] - 160.0).abs() < 0.5, "{load_ys:?}");
    }

    #[test]
    fn optimize_survives_cycles_and_disconnected() {
        // a <-> b feedback plus an island: back edge dropped, so the pair
        // stays in two adjacent columns instead of inflating.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 80.0)];
        let edges = [pe(0, 50.0, 1, 50.0), pe(1, 20.0, 0, 20.0)];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        assert_eq!(r.len(), 3);
        let max_col = r.iter().map(|e| e.1).max().unwrap();
        assert!(max_col <= 1, "cycle inflated columns: {r:?}");
    }

    #[test]
    fn optimize_undirected_edges_do_not_layer() {
        // Two loads sharing a net (no driver direction): stay in one column,
        // pulled adjacent by the barycenter.
        let nodes = [pn(0, 100.0), pn(1, 100.0)];
        let edges = [PlaceEdge {
            from: 0,
            from_dy: 50.0,
            to: 1,
            to_dy: 50.0,
            directed: false,
        }];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        assert_eq!(
            r[0].1, r[1].1,
            "undirected edge must not split columns: {r:?}"
        );
    }

    #[test]
    fn optimize_spreads_into_column_budget() {
        // Six unconnected blocks would all land in column 0 — with a budget
        // of 3 they spread across three columns instead of one tall stack.
        let nodes: Vec<PlaceNode> = (0..6).map(|i| pn(i, 100.0)).collect();
        let r = optimize_positions(&nodes, &[], 60.0, 3);
        let mut cols: Vec<i32> = r.iter().map(|e| e.1).collect();
        cols.sort_unstable();
        cols.dedup();
        assert_eq!(cols.len(), 3, "{r:?}");
    }

    #[test]
    fn optimize_centers_driver_on_fanout() {
        // The driver's output pin sits at the mean of the input pins it
        // feeds — fan spreads symmetrically, not bundled to one side.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 100.0), pn(3, 100.0)];
        let edges = [
            pe(0, 50.0, 1, 50.0),
            pe(0, 50.0, 2, 50.0),
            pe(0, 50.0, 3, 50.0),
        ];
        let r = optimize_positions(&nodes, &edges, 60.0, 0);
        let y_of = |id: i32| r.iter().find(|e| e.0 == id).unwrap().2;
        let load_pin_mean = (y_of(1) + y_of(2) + y_of(3)) / 3.0 + 50.0;
        assert!(
            (y_of(0) + 50.0 - load_pin_mean).abs() < 5.0,
            "driver not centered: {r:?}"
        );
    }

    #[test]
    fn optimize_respects_column_cap() {
        // Chain of four would want columns 0..3; cap at 2 folds the tail.
        let nodes = [pn(0, 100.0), pn(1, 100.0), pn(2, 100.0), pn(3, 100.0)];
        let edges = [
            pe(0, 50.0, 1, 50.0),
            pe(1, 50.0, 2, 50.0),
            pe(2, 50.0, 3, 50.0),
        ];
        let r = optimize_positions(&nodes, &edges, 60.0, 2);
        let max_col = r.iter().map(|e| e.1).max().unwrap();
        assert_eq!(max_col, 1, "cap ignored: {r:?}");
    }

    #[test]
    fn resolve_clear_y_no_obstacles_keeps_proposed() {
        assert_eq!(resolve_clear_y(0.0, 200.0, 100.0, 42.0, 30.0, &[]), 42.0);
    }

    #[test]
    fn resolve_clear_y_shoves_off_overlap() {
        let other = Rect {
            left: 0.0,
            top: 0.0,
            right: 200.0,
            bottom: 100.0,
        };
        let y = resolve_clear_y(0.0, 200.0, 100.0, 50.0, 30.0, &[other]);
        // Contract: no raw overlap with the other module. (The double-margin
        // candidate rejection means the fallback can yield half-gap
        // separation — quirk preserved from the C++ original.)
        assert!(y >= other.bottom || y + 100.0 <= other.top, "y={y}");
        assert_ne!(y, 50.0, "must have moved off the overlap");
    }

    // --- Property test: wires never cross module interiors --------------

    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
        fn range(&mut self, lo: i64, hi: i64) -> i64 {
            lo + (self.next() % (hi - lo + 1) as u64) as i64
        }
    }

    fn seg_crosses_rect(a: Point, b: Point, r: &Rect) -> bool {
        // Axis-aligned segments only (the router emits only H/V segments).
        const EPS: f64 = 1.0;
        if (a.y - b.y).abs() < 1e-9 {
            let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
            let y = a.y;
            return y > r.top + EPS
                && y < r.bottom - EPS
                && x1 > r.left + EPS
                && x0 < r.right - EPS
                && !(r.contains_strict(a.x, a.y, -1.0) || r.contains_strict(b.x, b.y, -1.0));
        }
        if (a.x - b.x).abs() < 1e-9 {
            let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
            let x = a.x;
            return x > r.left + EPS
                && x < r.right - EPS
                && y1 > r.top + EPS
                && y0 < r.bottom - EPS;
        }
        false
    }

    /// Random scenes built the way the app builds them (column-snapped
    /// modules, width capped below the pitch, vertical overlap resolved via
    /// resolve_clear_y, pins on module edges) must never route a wire
    /// through a module body.
    #[test]
    fn property_wires_avoid_module_interiors() {
        let mut rng = Lcg(0x5eed);
        for case in 0..200 {
            // Place 2–5 modules.
            let n_mods = rng.range(2, 5);
            let mut mods: Vec<Rect> = Vec::new();
            for _ in 0..n_mods {
                let col = rng.range(0, 2) as f64;
                let w = rng.range(200, 360) as f64; // capped below pitch-2*gutter
                let h = rng.range(80, 300) as f64;
                let left = col * P.column_pitch - w / 2.0;
                let y0 = rng.range(-400, 400) as f64;
                let y = resolve_clear_y(left, w, h, y0, 30.0, &mods);
                mods.push(Rect {
                    left,
                    top: y,
                    right: left + w,
                    bottom: y + h,
                });
            }
            // 1–4 nets between random module edge pins.
            let n_nets = rng.range(1, 4);
            let mut nets: Vec<Net> = Vec::new();
            for _ in 0..n_nets {
                let pin = |rng: &mut Lcg, mods: &[Rect]| -> Endpoint {
                    let m = &mods[rng.range(0, mods.len() as i64 - 1) as usize];
                    let right_edge = rng.range(0, 1) == 1;
                    let x = if right_edge { m.right } else { m.left };
                    let y = m.top + 20.0 + rng.range(0, (m.bottom - m.top - 40.0) as i64) as f64;
                    let col = (((m.left + m.right) / 2.0) / P.column_pitch).round() as i32;
                    Endpoint {
                        x,
                        y,
                        exits_right: right_edge,
                        col,
                    }
                };
                let driver = pin(&mut rng, &mods);
                let n_loads = rng.range(1, 2);
                let loads = (0..n_loads).map(|_| pin(&mut rng, &mods)).collect();
                nets.push(Net { driver, loads });
            }

            let r = plan_routes(&nets, &mods, P);
            for (wi, wire) in r.wires.iter().enumerate() {
                for seg in wire.windows(2) {
                    for (mi, m) in mods.iter().enumerate() {
                        assert!(
                            !seg_crosses_rect(seg[0], seg[1], m),
                            "case {case}: wire {wi} segment ({:?} → {:?}) crosses module {mi} {m:?}",
                            seg[0],
                            seg[1]
                        );
                    }
                }
            }
        }
    }
}
