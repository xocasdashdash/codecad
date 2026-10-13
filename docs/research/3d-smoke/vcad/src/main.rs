fn main() {
    for round in [false, true] {
        let t = std::time::Instant::now();
        let r = std::panic::catch_unwind(|| smoke_vcad::board(round));
        match r { Ok(s) => { let dt = t.elapsed(); let m = s.to_mesh(32); println!("round={round}: fidelity {:?} {dt:?}", s.fidelity()); println!("round={round}: {dt:?}, {} tris, vol {:?}", m.num_triangles(), vcad_kernel::vcad_kernel_booleans::mesh_signed_volume(&m)); }, Err(_) => println!("round={round}: panicked") }
    }
}
