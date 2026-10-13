use csgrs::adapter::mesh::TriangleMeshF64 as M;

pub fn board() -> M {
    let board = M::cuboid(600.0, 300.0, 18.0).unwrap();
    let hole = M::cylinder(5.0, 20.0, 64).unwrap().translate(50.0, 50.0, -1.0).unwrap();
    let dado = M::cuboid(602.0, 12.0, 7.0).unwrap().translate(-1.0, 144.0, 12.0).unwrap();
    board.difference(&hole).unwrap().difference(&dado).unwrap()
}

pub fn tri_count(m: &M) -> usize {
    m.graphics_mesh().unwrap().indices.len() / 3
}

#[no_mangle]
pub extern "C" fn smoke_run() -> u32 {
    tri_count(&board()) as u32
}
