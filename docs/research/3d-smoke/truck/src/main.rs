use truck_meshalgo::prelude::*;
fn main() {
    let t = std::time::Instant::now();
    let s = smoke_truck::board_with_hole_and_dado();
    let dt = t.elapsed();
    println!("faces in B-rep: {}", s.boundaries()[0].len());
    for tol in [1.0, 0.1, 0.01, 0.001] {
        let t2 = std::time::Instant::now();
        let m = s.triangulation(tol).to_polygon();
        let p = m.positions();
        let mut v = 0.0;
        for f in m.faces().triangle_iter() { let (a,b,c)=(p[f[0].pos],p[f[1].pos],p[f[2].pos]); v += a.to_vec().dot(b.to_vec().cross(c.to_vec()))/6.0; }
        println!("tol {tol}: {} triangles, mesh volume {v:.3} ({:?})", m.faces().len(), t2.elapsed());
    }
    println!("boolean time {:?}; exact expected 3195386.282", dt);
}
