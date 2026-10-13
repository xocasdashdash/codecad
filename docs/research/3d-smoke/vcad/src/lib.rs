use vcad_kernel::Solid;
pub fn board(round: bool) -> Solid {
    let mut b = Solid::cube(600.0, 300.0, 18.0);
    if round { b = b.fillet(3.0).expect("fillet"); }
    let hole = Solid::cylinder(5.0, 20.0, 64).translate(50.0, 50.0, -1.0);
    let dado = Solid::cube(602.0, 12.0, 7.0).translate(-1.0, 144.0, 12.0);
    b.difference(&hole).difference(&dado)
}
#[no_mangle]
pub extern "C" fn smoke_run() -> u32 { board(true).to_mesh(32).num_triangles() as u32 }
