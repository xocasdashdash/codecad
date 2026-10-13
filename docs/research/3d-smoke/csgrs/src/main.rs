use csgrs::adapter::mesh::TriangleMeshF64 as M;
fn main() {
    let board = M::cuboid(600.0, 300.0, 18.0).unwrap();
    let dado = M::cuboid(602.0, 12.0, 7.0).unwrap().translate(-1.0, 144.0, 12.0).unwrap();
    let t = std::time::Instant::now();
    match board.difference(&dado) { Ok(m) => println!("box-box ok {:?} {} tris", t.elapsed(), smoke_csgrs::tri_count(&m)), Err(e) => println!("box-box err {e:?}") }
    for seg in [8usize, 16, 32, 64] {
        let hole = M::cylinder(5.0, 20.0, seg).unwrap().translate(50.0, 50.0, -1.0).unwrap();
        let t = std::time::Instant::now();
        match board.difference(&hole) { Ok(m) => println!("box-cyl{seg} ok {:?} {} tris", t.elapsed(), smoke_csgrs::tri_count(&m)), Err(e) => println!("box-cyl{seg} err {e:?}") }
    }
}
