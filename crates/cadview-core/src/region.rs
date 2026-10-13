//! Boolean operations on polygonal regions (via the pure-Rust `i_overlay`
//! crate, which also compiles to wasm32).
//!
//! Regions are lists of closed rings; within one operand rings combine
//! with the even-odd rule, so `[outer, hole]` describes a ring with a hole.
//! Results are polygonal: curved input must be sampled first. This is a
//! construction aid (e.g. clip a hatch boundary, cut a window out of a
//! wall outline); the abstract-geometry rule still applies to entities.

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;

/// A polygon with holes: outer ring (CCW) and holes (CW).
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub outer: Vec<[f64; 2]>,
    pub holes: Vec<Vec<[f64; 2]>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    Intersect,
    Union,
    Subtract,
    Xor,
}

impl BoolOp {
    pub fn parse(s: Option<&str>) -> Result<Self, String> {
        match s.unwrap_or("intersect") {
            "intersect" | "intersection" | "and" => Ok(BoolOp::Intersect),
            "union" | "or" | "add" => Ok(BoolOp::Union),
            "subtract" | "difference" | "diff" | "minus" => Ok(BoolOp::Subtract),
            "xor" => Ok(BoolOp::Xor),
            o => Err(format!(
                "unknown boolean op '{o}' (intersect|union|subtract|xor)"
            )),
        }
    }
}

/// Normalise a ring to counter-clockwise orientation.
fn ccw(ring: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut r = ring.to_vec();
    let signed: f64 = (0..r.len())
        .map(|i| {
            let (a, b) = (r[i], r[(i + 1) % r.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if signed < 0.0 {
        r.reverse();
    }
    r
}

/// Apply a boolean operation to two regions given as ring lists.
///
/// Every ring is a filled shape: the rings of one side are unioned
/// (non-zero winding after normalising orientation), so overlapping
/// rings merge instead of cancelling. Holes are made by subtracting.
pub fn boolean(subject: &[Vec<[f64; 2]>], clip: &[Vec<[f64; 2]>], op: BoolOp) -> Vec<Region> {
    let subj: Vec<Vec<[f64; 2]>> = subject
        .iter()
        .filter(|r| r.len() >= 3)
        .map(|r| ccw(r))
        .collect();
    let clip: Vec<Vec<[f64; 2]>> = clip
        .iter()
        .filter(|r| r.len() >= 3)
        .map(|r| ccw(r))
        .collect();
    let rule = match op {
        BoolOp::Intersect => OverlayRule::Intersect,
        BoolOp::Union => OverlayRule::Union,
        BoolOp::Subtract => OverlayRule::Difference,
        BoolOp::Xor => OverlayRule::Xor,
    };
    let shapes = subj.overlay(&clip, rule, FillRule::NonZero);
    shapes
        .into_iter()
        .filter_map(|mut contours| {
            if contours.is_empty() {
                return None;
            }
            let outer = contours.remove(0);
            Some(Region {
                outer,
                holes: contours,
            })
        })
        .collect()
}

/// Unsigned area of a region (outer minus holes).
pub fn region_area(r: &Region) -> f64 {
    fn ring(p: &[[f64; 2]]) -> f64 {
        let n = p.len();
        let mut a = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            a += p[i][0] * p[j][1] - p[j][0] * p[i][1];
        }
        (a / 2.0).abs()
    }
    ring(&r.outer) - r.holes.iter().map(|h| ring(h)).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(x: f64, y: f64, s: f64) -> Vec<[f64; 2]> {
        vec![[x, y], [x + s, y], [x + s, y + s], [x, y + s]]
    }

    #[test]
    fn intersect_union_subtract() {
        let a = vec![sq(0., 0., 4.)];
        let b = vec![sq(2., 2., 4.)];
        let i = boolean(&a, &b, BoolOp::Intersect);
        assert_eq!(i.len(), 1);
        assert!((region_area(&i[0]) - 4.0).abs() < 1e-9);
        let u = boolean(&a, &b, BoolOp::Union);
        assert_eq!(u.len(), 1);
        assert!((region_area(&u[0]) - 28.0).abs() < 1e-9);
        let d = boolean(&a, &b, BoolOp::Subtract);
        assert!((d.iter().map(region_area).sum::<f64>() - 12.0).abs() < 1e-9);
        let x = boolean(&a, &b, BoolOp::Xor);
        assert!((x.iter().map(region_area).sum::<f64>() - 24.0).abs() < 1e-9);
    }

    #[test]
    fn subtract_inner_square_makes_hole() {
        let a = vec![sq(0., 0., 10.)];
        let b = vec![sq(3., 3., 2.)];
        let d = boolean(&a, &b, BoolOp::Subtract);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].holes.len(), 1);
        assert!((region_area(&d[0]) - 96.0).abs() < 1e-9);
    }

    #[test]
    fn overlapping_subject_rings_merge() {
        // Four external walls of a 9000 x 6000 building, 300 thick: the
        // corner squares are covered twice and must stay solid.
        let r = |x0: f64, y0: f64, x1: f64, y1: f64| vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
        let walls = vec![
            r(0., 0., 9000., 300.),
            r(0., 5700., 9000., 6000.),
            r(0., 0., 300., 6000.),
            r(8700., 6000., 9000., 0.), // clockwise on purpose
        ];
        let u = boolean(&walls, &[], BoolOp::Union);
        assert_eq!(u.len(), 1);
        assert_eq!(u[0].holes.len(), 1);
        assert!((region_area(&u[0]) - (9000. * 6000. - 8400. * 5400.)).abs() < 1e-6);
    }

    #[test]
    fn disjoint_intersection_is_empty() {
        let a = vec![sq(0., 0., 1.)];
        let b = vec![sq(5., 5., 1.)];
        assert!(boolean(&a, &b, BoolOp::Intersect).is_empty());
        assert!(BoolOp::parse(Some("nope")).is_err());
    }
}
