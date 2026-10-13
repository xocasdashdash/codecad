//! B-spline construction helpers: clamped knot vectors for control-point
//! splines and global interpolation through fit points (Piegl & Tiller,
//! The NURBS Book, A9.1, chord-length parameters, averaged knots).

use kurbo::Point;

/// Clamped uniform knot vector for `n` control points of degree `p`.
pub fn clamped_knots(n: usize, p: usize) -> Vec<f64> {
    let mut k = vec![0.0; p + 1];
    let interior = n.saturating_sub(p + 1);
    for i in 1..=interior {
        k.push(i as f64 / (interior + 1) as f64);
    }
    k.extend(std::iter::repeat_n(1.0, p + 1));
    k
}

/// Cox-de Boor basis function N_{i,p}(u).
pub fn basis(i: usize, p: usize, u: f64, knots: &[f64]) -> f64 {
    let m = knots.len() - 1;
    // Special case: u at the very end belongs to the last basis function.
    if (u - knots[m]).abs() < 1e-14 {
        let n = m - p - 1;
        return if i == n { 1.0 } else { 0.0 };
    }
    if p == 0 {
        return if knots[i] <= u && u < knots[i + 1] {
            1.0
        } else {
            0.0
        };
    }
    let mut v = 0.0;
    let d1 = knots[i + p] - knots[i];
    if d1 > 0.0 {
        v += (u - knots[i]) / d1 * basis(i, p - 1, u, knots);
    }
    let d2 = knots[i + p + 1] - knots[i + 1];
    if d2 > 0.0 {
        v += (knots[i + p + 1] - u) / d2 * basis(i + 1, p - 1, u, knots);
    }
    v
}

/// Evaluate a B-spline at parameter `u`.
pub fn eval(p: usize, knots: &[f64], ctrl: &[Point], u: f64) -> Point {
    let mut x = 0.0;
    let mut y = 0.0;
    for (i, c) in ctrl.iter().enumerate() {
        let b = basis(i, p, u, knots);
        x += b * c.x;
        y += b * c.y;
    }
    Point::new(x, y)
}

/// Interpolating B-spline through `pts`. Returns (degree, knots, control points).
pub fn interpolate(pts: &[Point], degree: usize) -> Result<(usize, Vec<f64>, Vec<Point>), String> {
    // Drop consecutive duplicates (they would give zero-length chords).
    let mut q: Vec<Point> = Vec::with_capacity(pts.len());
    for p in pts {
        if q.last().is_none_or(|l: &Point| (*l - *p).hypot() > 1e-12) {
            q.push(*p);
        }
    }
    if q.len() < 2 {
        return Err("spline needs at least 2 distinct points".into());
    }
    let n = q.len() - 1;
    let p = degree.clamp(1, n);
    let total: f64 = q.windows(2).map(|w| (w[1] - w[0]).hypot()).sum();
    let mut params = vec![0.0; n + 1];
    for k in 1..=n {
        params[k] = params[k - 1] + (q[k] - q[k - 1]).hypot() / total;
    }
    params[n] = 1.0;
    let mut knots = vec![0.0; p + 1];
    for j in 1..=(n - p) {
        knots.push(params[j..j + p].iter().sum::<f64>() / p as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, p + 1));

    // Solve A * P = Q with partial-pivot Gaussian elimination.
    let size = n + 1;
    let mut a = vec![vec![0.0; size]; size];
    for (k, row) in a.iter_mut().enumerate() {
        for (i, v) in row.iter_mut().enumerate() {
            *v = basis(i, p, params[k], &knots);
        }
    }
    let mut bx: Vec<f64> = q.iter().map(|p| p.x).collect();
    let mut by: Vec<f64> = q.iter().map(|p| p.y).collect();
    for col in 0..size {
        let piv = (col..size)
            .max_by(|&i, &j| {
                a[i][col]
                    .abs()
                    .partial_cmp(&a[j][col].abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("non-empty range");
        if a[piv][col].abs() < 1e-14 {
            return Err("spline interpolation system is singular".into());
        }
        a.swap(col, piv);
        bx.swap(col, piv);
        by.swap(col, piv);
        for r in col + 1..size {
            let f = a[r][col] / a[col][col];
            if f == 0.0 {
                continue;
            }
            let (top, bottom) = a.split_at_mut(r);
            for (x, y) in bottom[0][col..].iter_mut().zip(&top[col][col..]) {
                *x -= f * y;
            }
            bx[r] -= f * bx[col];
            by[r] -= f * by[col];
        }
    }
    let mut cx = vec![0.0; size];
    let mut cy = vec![0.0; size];
    for r in (0..size).rev() {
        let mut sx = bx[r];
        let mut sy = by[r];
        for c in r + 1..size {
            sx -= a[r][c] * cx[c];
            sy -= a[r][c] * cy[c];
        }
        cx[r] = sx / a[r][r];
        cy[r] = sy / a[r][r];
    }
    let ctrl = cx
        .into_iter()
        .zip(cy)
        .map(|(x, y)| Point::new(x, y))
        .collect();
    Ok((p, knots, ctrl))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamped_knot_vector_shape() {
        let k = clamped_knots(5, 3);
        assert_eq!(k.len(), 5 + 3 + 1);
        assert_eq!(&k[..4], &[0.0; 4]);
        assert_eq!(&k[5..], &[1.0; 4]);
    }

    #[test]
    fn interpolation_passes_through_fit_points() {
        let pts = vec![
            Point::new(0., 0.),
            Point::new(10., 5.),
            Point::new(20., -3.),
            Point::new(35., 8.),
            Point::new(50., 0.),
        ];
        let (p, knots, ctrl) = interpolate(&pts, 3).unwrap();
        assert_eq!(p, 3);
        assert_eq!(knots.len(), ctrl.len() + p + 1);
        // Recompute chord-length parameters and check every fit point.
        let total: f64 = pts.windows(2).map(|w| (w[1] - w[0]).hypot()).sum();
        let mut u = 0.0;
        for (k, q) in pts.iter().enumerate() {
            if k > 0 {
                u += (pts[k] - pts[k - 1]).hypot() / total;
            }
            let e = eval(p, &knots, &ctrl, u.min(1.0));
            assert!((e - *q).hypot() < 1e-6, "point {k}: {e:?} vs {q:?}");
        }
    }

    #[test]
    fn two_points_degrade_to_line_and_duplicates_error() {
        let (p, _, ctrl) = interpolate(&[Point::new(0., 0.), Point::new(1., 1.)], 3).unwrap();
        assert_eq!(p, 1);
        assert_eq!(ctrl.len(), 2);
        assert!(interpolate(&[Point::new(1., 1.), Point::new(1., 1.)], 3).is_err());
    }
}
