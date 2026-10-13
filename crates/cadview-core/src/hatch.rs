use crate::geo;
use crate::types::*;
use kurbo::{Line, Point};
use std::f64::consts::PI;

/// Clip a line (p0..p1) to a convex or concave polygon.
/// Returns segments of the line that are inside the polygon.
/// Generate parallel hatch fill lines clipped to a boundary polygon.
/// Returns Line shapes. `angle` in radians, `spacing` in drawing units.
pub(crate) fn generate_hatch_lines(boundary: &[Point], angle: f64, spacing: f64) -> Vec<Shape> {
    if boundary.len() < 3 || spacing <= 0.0 {
        return Vec::new();
    }

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let perp_dists: Vec<f64> = boundary
        .iter()
        .map(|p| -p.x * sin_a + p.y * cos_a)
        .collect();
    let min_d = perp_dists.iter().cloned().fold(f64::MAX, f64::min);
    let max_d = perp_dists.iter().cloned().fold(f64::MIN, f64::max);

    let (bx0, by0, bx1, by1) = geo::bounds_of(boundary);
    let diag = ((bx1 - bx0).powi(2) + (by1 - by0).powi(2)).sqrt();

    let mut shapes = Vec::new();
    let mut d = min_d + spacing;
    while d < max_d {
        let base = Point::new(-d * sin_a, d * cos_a);
        let dir = Point::new(cos_a, sin_a);
        let p0 = Point::new(base.x - diag * dir.x, base.y - diag * dir.y);
        let p1 = Point::new(base.x + diag * dir.x, base.y + diag * dir.y);

        for (ca, cb) in clip_line_to_polygon(p0, p1, boundary) {
            shapes.push(Shape::Line(Line::new(ca, cb)));
        }
        d += spacing;
    }
    shapes
}

/// Generate hatch fill from DWG hatch pattern data.
///
/// Each HatchPatternLine defines a family of parallel lines:
/// - `angle`: direction of the lines (radians)
/// - `base_point`: origin for the line family
/// - `offset.x`: along-line shift between consecutive rows (stagger)
/// - `offset.y`: perpendicular spacing between rows
/// - `dash_lengths`: dash pattern (positive=draw, negative=gap, empty=solid)
///
/// Generate fill lines for a DWG hatch pattern clipped to a boundary polygon.
///
/// The DWG binary format stores RESOLVED per-line values:
///   - line angles already include pattern_angle (pre-baked by AutoCAD)
///   - offsets are in world/OCS space (pre-rotated by line angle)
///   - dashes and offsets are NOT scaled by pattern_scale (we apply it here)
pub fn generate_dwg_hatch_fill(
    boundary: &[Point],
    pattern: &acadrust::entities::HatchPattern,
    _pattern_angle: f64, // already baked into per-line angles by AutoCAD
    pattern_scale: f64,
    is_double: bool,
) -> Vec<Shape> {
    if boundary.len() < 3 {
        return Vec::new();
    }
    let scale = if pattern_scale > 0.0 {
        pattern_scale
    } else {
        1.0
    };
    let (bx0, by0, bx1, by1) = geo::bounds_of(boundary);
    let diag = ((bx1 - bx0).powi(2) + (by1 - by0).powi(2)).sqrt();

    let mut shapes = Vec::new();

    for pat_line in &pattern.lines {
        let angle = pat_line.angle;

        // Offsets are in world/OCS space (pre-rotated by AutoCAD).
        // Apply pattern_scale but use directly as the row step vector.
        let off_x = pat_line.offset.x * scale;
        let off_y = pat_line.offset.y * scale;
        let off_len = (off_x * off_x + off_y * off_y).sqrt();
        if off_len < 1e-6 {
            continue;
        }
        // Signed perpendicular step per row: project offset onto line normal.
        // Sign matters for mapping row indices to perpendicular distances.
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let signed_spacing = -off_x * sin_a + off_y * cos_a;
        if signed_spacing.abs() < 1e-6 {
            continue;
        }
        let base = Point::new(pat_line.base_point.x * scale, pat_line.base_point.y * scale);

        // Direction along the line and perpendicular
        let dir = Point::new(cos_a, sin_a);
        let perp = Point::new(-sin_a, cos_a);

        // Project boundary onto perpendicular to find range of parallel lines
        let perp_dists: Vec<f64> = boundary
            .iter()
            .map(|p| (p.x - base.x) * perp.x + (p.y - base.y) * perp.y)
            .collect();
        let min_d = perp_dists.iter().cloned().fold(f64::MAX, f64::min);
        let max_d = perp_dists.iter().cloned().fold(f64::MIN, f64::max);

        // Row N is at perpendicular distance N * signed_spacing from base.
        // Solve for row range: min_d <= N * signed_spacing <= max_d.
        // When signed_spacing < 0, the inequality flips.
        let a = min_d / signed_spacing;
        let b = max_d / signed_spacing;
        let row_start = a.min(b).floor() as i64;
        let row_end = a.max(b).ceil() as i64;

        let has_dashes = !pat_line.dash_lengths.is_empty();

        for row in row_start..=row_end {
            // Step by the FULL offset vector for each row.
            // This naturally handles both spacing and stagger.
            let row_base = Point::new(base.x + row as f64 * off_x, base.y + row as f64 * off_y);

            // Extend line far enough to cross the entire boundary
            let p0 = Point::new(row_base.x - diag * dir.x, row_base.y - diag * dir.y);
            let p1 = Point::new(row_base.x + diag * dir.x, row_base.y + diag * dir.y);

            // Clip to boundary
            let clipped = clip_line_to_polygon(p0, p1, boundary);

            if !has_dashes {
                // Solid lines
                for (ca, cb) in clipped {
                    shapes.push(Shape::Line(Line::new(ca, cb)));
                }
            } else {
                // Dashed lines: break each clipped segment into dash sub-segments
                for (ca, cb) in clipped {
                    let seg_dx = cb.x - ca.x;
                    let seg_dy = cb.y - ca.y;
                    let seg_len = (seg_dx * seg_dx + seg_dy * seg_dy).sqrt();
                    if seg_len < 1e-6 {
                        continue;
                    }
                    let ux = seg_dx / seg_len;
                    let uy = seg_dy / seg_len;

                    // Compute dash phase: distance along line from row_base to clip start.
                    // The offset vector already includes any stagger (along-line shift),
                    // so the phase naturally aligns across rows.
                    let t_start = (ca.x - row_base.x) * dir.x + (ca.y - row_base.y) * dir.y;

                    // Total dash pattern length
                    let pattern_len: f64 =
                        pat_line.dash_lengths.iter().map(|d| d.abs() * scale).sum();
                    if pattern_len < 1e-6 {
                        continue;
                    }

                    // Phase: where in the pattern does this segment start?
                    let phase = ((t_start % pattern_len) + pattern_len) % pattern_len;

                    // Walk along the segment applying the dash pattern
                    let mut pos = 0.0_f64; // position along segment
                    let mut pat_pos = phase; // position within pattern

                    while pos < seg_len {
                        // Find current dash element
                        let mut acc = 0.0;
                        let mut dash_idx = 0;
                        for (i, d) in pat_line.dash_lengths.iter().enumerate() {
                            let dl = d.abs() * scale;
                            if acc + dl > pat_pos {
                                dash_idx = i;
                                break;
                            }
                            acc += dl;
                            dash_idx = i + 1;
                        }
                        if dash_idx >= pat_line.dash_lengths.len() {
                            // Wrap around
                            pat_pos %= pattern_len;
                            continue;
                        }

                        let dash_val = pat_line.dash_lengths[dash_idx];
                        let dash_len = dash_val.abs() * scale;
                        let remaining_in_dash = dash_len - (pat_pos - acc);
                        let remaining_in_seg = seg_len - pos;
                        let step = remaining_in_dash.min(remaining_in_seg);

                        if dash_val >= 0.0 {
                            // Pen down: draw (0.0 = dot, treated as tiny segment)
                            let draw_len = if dash_val == 0.0 { 0.5 * scale } else { step };
                            let draw_len = draw_len.min(remaining_in_seg);
                            let sx = ca.x + pos * ux;
                            let sy = ca.y + pos * uy;
                            let ex = ca.x + (pos + draw_len) * ux;
                            let ey = ca.y + (pos + draw_len) * uy;
                            shapes.push(Shape::Line(Line::new(
                                Point::new(sx, sy),
                                Point::new(ex, ey),
                            )));
                        }
                        // else: pen up (gap), skip

                        pos += step;
                        pat_pos += step;
                        if pat_pos >= pattern_len {
                            pat_pos -= pattern_len;
                        }
                    }
                }
            }
        }
    }

    if is_double && !pattern.lines.is_empty() {
        let base_angle = pattern.lines[0].angle;
        let spacing = pattern.lines[0].offset.y.abs() * scale;
        if spacing > 1e-6 {
            let fill = generate_hatch_lines(boundary, base_angle + PI / 2.0, spacing);
            shapes.extend(fill);
        }
    }

    shapes
}

pub(crate) fn clip_line_to_polygon(p0: Point, p1: Point, polygon: &[Point]) -> Vec<(Point, Point)> {
    clip_line_to_rings(p0, p1, std::slice::from_ref(&polygon.to_vec()))
}

/// Clip the infinite line through p0, p1 against a region made of one or
/// more closed rings (outer boundary + holes, or several islands) using
/// the even-odd rule. Returns the inside spans ordered along p0->p1.
///
/// Crossings are counted with a half-open rule (an edge crosses when its
/// endpoints are strictly on opposite sides, with "on the line" treated as
/// below). That makes a scanline through a vertex count 0 or 2 crossings
/// for a touching corner and exactly 1 for a pass-through vertex, so no
/// de-duplication of intersection points is needed. Merging nearby
/// intersection parameters (the old approach) dropped one side of a pair
/// and mis-paired every following span, which lost scanlines around holes
/// and at corner-touching lines.
pub fn clip_line_to_rings(p0: Point, p1: Point, rings: &[Vec<Point>]) -> Vec<(Point, Point)> {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-24 {
        return Vec::new();
    }
    let len = len2.sqrt();
    // Unit normal of the scanline; side(q) = signed distance from the line.
    let (nx, ny) = (-dy / len, dx / len);
    let side = |q: Point| (q.x - p0.x) * nx + (q.y - p0.y) * ny;

    let mut ts = Vec::new();
    for ring in rings {
        let n = ring.len();
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let a = ring[i];
            let b = ring[(i + 1) % n];
            let sa = side(a);
            let sb = side(b);
            if (sa > 0.0) == (sb > 0.0) {
                continue;
            }
            let f = sa / (sa - sb);
            let q = Point::new(a.x + (b.x - a.x) * f, a.y + (b.y - a.y) * f);
            ts.push(((q.x - p0.x) * dx + (q.y - p0.y) * dy) / len2);
        }
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut segments = Vec::new();
    for &[ta, tb] in ts.as_chunks::<2>().0 {
        if (tb - ta) * len <= 1e-9 {
            continue;
        }
        segments.push((
            Point::new(p0.x + ta * dx, p0.y + ta * dy),
            Point::new(p0.x + tb * dx, p0.y + tb * dy),
        ));
    }
    segments
}

/// Parallel hatch lines over a region (outer ring + holes, even-odd).
/// Lines are anchored at multiples of `spacing` from the origin so that
/// adjacent hatched regions line up. `angle` in radians.
pub fn hatch_lines_for_rings(
    rings: &[Vec<Point>],
    angle: f64,
    spacing: f64,
) -> Vec<(Point, Point)> {
    let all: Vec<Point> = rings.iter().flatten().copied().collect();
    if all.len() < 3 || spacing <= 0.0 || !spacing.is_finite() {
        return Vec::new();
    }
    let (cos_a, sin_a) = (angle.cos(), angle.sin());
    let perp: Vec<f64> = all.iter().map(|p| -p.x * sin_a + p.y * cos_a).collect();
    let min_d = perp.iter().cloned().fold(f64::MAX, f64::min);
    let max_d = perp.iter().cloned().fold(f64::MIN, f64::max);
    let (bx0, by0, bx1, by1) = geo::bounds_of(&all);
    let diag = ((bx1 - bx0).powi(2) + (by1 - by0).powi(2)).sqrt() + 1.0;
    // Centre of the region projected on the hatch direction, so the scan
    // segment always spans the whole region.
    let along_c = ((bx0 + bx1) / 2.0) * cos_a + ((by0 + by1) / 2.0) * sin_a;

    let mut out = Vec::new();
    let mut k = (min_d / spacing).floor() as i64;
    if (k as f64) * spacing <= min_d + 1e-9 * spacing {
        k += 1;
    }
    // Safety valve against absurd spacing/size ratios.
    let max_lines = 200_000;
    let mut count = 0;
    loop {
        let d = k as f64 * spacing;
        if d >= max_d - 1e-9 * spacing || count > max_lines {
            break;
        }
        let base = Point::new(-d * sin_a + along_c * cos_a, d * cos_a + along_c * sin_a);
        let a = Point::new(base.x - diag * cos_a, base.y - diag * sin_a);
        let b = Point::new(base.x + diag * cos_a, base.y + diag * sin_a);
        out.extend(clip_line_to_rings(a, b, rings));
        k += 1;
        count += 1;
    }
    out
}

/// Extract the start/end points of an entity for connectivity checks.
pub(crate) fn entity_endpoints(shape: &Shape) -> Vec<Point> {
    match shape {
        Shape::Line(l) => vec![l.p0, l.p1],
        Shape::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            let p0 = Point::new(
                center.x + radius * start_angle.cos(),
                center.y + radius * start_angle.sin(),
            );
            let p1 = Point::new(
                center.x + radius * end_angle.cos(),
                center.y + radius * end_angle.sin(),
            );
            if geo::distance(p0, p1) > 1e-6 {
                vec![p0, p1]
            } else {
                vec![p0]
            }
        }
        Shape::Polyline { points, .. } => {
            let mut result = Vec::new();
            if let Some(first) = points.first() {
                result.push(*first);
            }
            if let Some(last) = points.last() {
                if result.is_empty() || geo::distance(result[0], *last) > 1e-6 {
                    result.push(*last);
                }
            }
            result
        }
        _ => Vec::new(),
    }
}
