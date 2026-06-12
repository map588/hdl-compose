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
    if exits_right { 2 * col + 1 } else { 2 * col - 1 }
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
        GutterInfo { safe_min: -1e18, safe_max: 1e18, nets_using: 0 }
    }
}

/// Tighten each gutter's safe X window so every wire keeps at least
/// `stub_min` of straight stub at its pin, and count how many nets share
/// each gutter (lane density).
fn build_gutter_info(nets: &[Net], p: Params) -> HashMap<i32, GutterInfo> {
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
    info
}

/// Pick the X for the next lane in gutter `idx`. Zigzag fan (0, +1, -1,
/// +2, -2, …) around the natural center; squeezes the step when the gutter
/// is bounded on both sides.
fn allocate_lane_x(
    idx: i32,
    info: &HashMap<i32, GutterInfo>,
    counter: &mut HashMap<i32, i32>,
    p: Params,
) -> f64 {
    let gi = info.get(&idx).copied().unwrap_or_default();
    let natural_x = gutter_center_x(idx, p.column_pitch);
    let bounded_left = gi.safe_min > -1e17;
    let bounded_right = gi.safe_max < 1e17;
    let n = *counter.get(&idx).unwrap_or(&0);
    counter.insert(idx, n + 1);

    let zigzag = |n: i32, step: f64| -> f64 {
        if n == 0 {
            return 0.0;
        }
        let sign = if n % 2 == 1 { 1.0 } else { -1.0 };
        let mag = ((n + 1) / 2) as f64;
        sign * mag * step
    };

    if bounded_left && bounded_right {
        if gi.safe_min > gi.safe_max {
            // Over-constrained gutter: pins on both sides demand more room
            // than exists (shouldn't happen with instance width capped below
            // the column pitch, but guard anyway). Split the violation
            // instead of collapsing every lane onto one bound.
            return (gi.safe_min + gi.safe_max) / 2.0;
        }
        let w = gi.safe_max - gi.safe_min;
        let center = (gi.safe_min + gi.safe_max) / 2.0;
        let lane_slots = gi.nets_using.max(1) as f64;
        let step = p.lane_step.min(w / lane_slots);
        let gx = center + zigzag(n, step);
        return gx.clamp(gi.safe_min, gi.safe_max);
    }
    if bounded_right {
        return gi.safe_max - n as f64 * p.lane_step;
    }
    if bounded_left {
        return gi.safe_min + n as f64 * p.lane_step;
    }
    natural_x + zigzag(n, p.lane_step)
}

/// Shift the bridge Y off any module body overlapping its X range.
fn adjust_bridge_y(preferred: f64, bx_min: f64, bx_max: f64, obstacles: &[Rect]) -> f64 {
    const MARGIN: f64 = 8.0;
    let blocking: Vec<(f64, f64)> = obstacles
        .iter()
        .filter(|r| !(r.right < bx_min || r.left > bx_max))
        .map(|r| (r.top - MARGIN, r.bottom + MARGIN))
        .collect();
    let in_blocked = |y: f64| blocking.iter().any(|b| y >= b.0 && y <= b.1);
    if !in_blocked(preferred) {
        return preferred;
    }
    let mut candidates: Vec<f64> = Vec::new();
    for b in &blocking {
        candidates.push(b.0 - 1.0);
        candidates.push(b.1 + 1.0);
    }
    candidates.sort_by(|a, b| {
        (a - preferred)
            .abs()
            .partial_cmp(&(b - preferred).abs())
            .unwrap()
    });
    for y in candidates {
        if !in_blocked(y) {
            return y;
        }
    }
    preferred
}

/// Route every net. `nets` must be in a stable order (the caller sorts by
/// net key) — lane assignment depends on allocation order.
pub fn plan_routes(nets: &[Net], obstacles: &[Rect], p: Params) -> RouteResult {
    let info = build_gutter_info(nets, p);
    let mut counter: HashMap<i32, i32> = HashMap::new();
    let mut result = RouteResult::default();

    for (net_idx, net) in nets.iter().enumerate() {
        let driver = net.driver;
        let g_d = gutter_index(driver.col, driver.exits_right);
        let g_l: Vec<i32> = net
            .loads
            .iter()
            .map(|l| gutter_index(l.col, l.exits_right))
            .collect();
        let mut unique_g: Vec<i32> = g_l.clone();
        unique_g.push(g_d);
        unique_g.sort_unstable();
        unique_g.dedup();

        let mut dot_points: Vec<Point> = Vec::new();

        if unique_g.len() == 1 {
            // Everything shares one gutter: stub → vertical trunk → stub.
            let gx = allocate_lane_x(g_d, &info, &mut counter, p);
            for l in &net.loads {
                result.wires.push(vec![
                    Point { x: driver.x, y: driver.y },
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
        } else {
            // Bridge route. Bridge Y = mean of all pin Ys, shifted off any
            // module body in its X range.
            let sum_y: f64 = driver.y + net.loads.iter().map(|l| l.y).sum::<f64>();
            let mut hy = sum_y / (1 + net.loads.len()) as f64;
            let mut gx_for_idx: HashMap<i32, f64> = HashMap::new();
            for &idx in &unique_g {
                gx_for_idx.insert(idx, allocate_lane_x(idx, &info, &mut counter, p));
            }
            let dgx = gx_for_idx[&g_d];

            let mut bx_min = f64::INFINITY;
            let mut bx_max = f64::NEG_INFINITY;
            for &v in gx_for_idx.values() {
                bx_min = bx_min.min(v);
                bx_max = bx_max.max(v);
            }
            hy = adjust_bridge_y(hy, bx_min, bx_max, obstacles);

            for (i, l) in net.loads.iter().enumerate() {
                let lgx = gx_for_idx[&g_l[i]];
                let mut wp = vec![
                    Point { x: driver.x, y: driver.y },
                    Point { x: dgx, y: driver.y },
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
        }

        // Dedupe per net on rounded coordinates.
        let mut seen: HashSet<(i64, i64)> = HashSet::new();
        for pt in dot_points {
            let key = (pt.x.round() as i64, pt.y.round() as i64);
            if seen.insert(key) {
                result.dots.push(Dot { x: pt.x, y: pt.y, net: net_idx });
            }
        }
    }
    result
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
        !blocking
            .iter()
            .any(|b| b.left < probe_right && b.right > probe_left && b.top < bottom && b.bottom > top)
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
    blocking
        .iter()
        .map(|b| b.bottom)
        .fold(proposed_y, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: Params = Params { column_pitch: 480.0, lane_step: 12.0, stub_min: 40.0 };

    fn ep(x: f64, y: f64, exits_right: bool, col: i32) -> Endpoint {
        Endpoint { x, y, exits_right, col }
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
    fn two_nets_in_one_gutter_get_distinct_lanes() {
        let nets = [
            Net { driver: ep(180.0, 100.0, true, 0), loads: vec![ep(300.0, 150.0, false, 1)] },
            Net { driver: ep(180.0, 200.0, true, 0), loads: vec![ep(300.0, 250.0, false, 1)] },
        ];
        let r = plan_routes(&nets, &[], P);
        assert_ne!(r.wires[0][1].x, r.wires[1][1].x, "lanes must not coincide");
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
        let block = Rect { left: 100.0, top: 150.0, right: 600.0, bottom: 250.0 };
        let r = plan_routes(&nets, &[block], P);
        let hy = r.wires[0][2].y; // bridge Y
        assert!(
            !(hy >= block.top - 8.0 && hy <= block.bottom + 8.0),
            "bridge y {hy} sits inside blocked band"
        );
    }

    #[test]
    fn resolve_clear_y_no_obstacles_keeps_proposed() {
        assert_eq!(resolve_clear_y(0.0, 200.0, 100.0, 42.0, 30.0, &[]), 42.0);
    }

    #[test]
    fn resolve_clear_y_shoves_off_overlap() {
        let other = Rect { left: 0.0, top: 0.0, right: 200.0, bottom: 100.0 };
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
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
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
                mods.push(Rect { left, top: y, right: left + w, bottom: y + h });
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
                    Endpoint { x, y, exits_right: right_edge, col }
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
