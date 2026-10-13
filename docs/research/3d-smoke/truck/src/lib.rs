use truck_meshalgo::prelude::*;
use truck_modeling::*;

/// A 600x300x18 mm shelf board with a 10 mm dowel hole drilled through
/// (box minus cylinder) and a 12x6 dado groove (box minus box).
pub fn board_with_hole_and_dado() -> Solid {
    let v = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let e = builder::tsweep(&v, Vector3::new(600.0, 0.0, 0.0));
    let f = builder::tsweep(&e, Vector3::new(0.0, 300.0, 0.0));
    let board: Solid = builder::tsweep(&f, Vector3::new(0.0, 0.0, 18.0));

    // dowel hole: circle in z=-1 plane, radius 5, at (50,50), extruded 20
    let p = builder::vertex(Point3::new(55.0, 50.0, -1.0));
    let circ = builder::rsweep(&p, Point3::new(50.0, 50.0, -1.0), Vector3::unit_z(), Rad(7.0));
    let disk = builder::try_attach_plane(&[circ]).unwrap();
    let mut cyl: Solid = builder::tsweep(&disk, Vector3::new(0.0, 0.0, 20.0));
    cyl.not();

    // dado: 12 wide (y 144..156), 6 deep from top (z 12..19), full length
    let v = builder::vertex(Point3::new(-1.0, 144.0, 12.0));
    let e = builder::tsweep(&v, Vector3::new(602.0, 0.0, 0.0));
    let f = builder::tsweep(&e, Vector3::new(0.0, 12.0, 0.0));
    let mut dado: Solid = builder::tsweep(&f, Vector3::new(0.0, 0.0, 7.0));
    dado.not();

    let s = truck_shapeops::and(&board, &cyl, 0.05).expect("boolean hole failed");
    truck_shapeops::and(&s, &dado, 0.05).expect("boolean dado failed")
}

pub fn tri_count(s: &Solid, tol: f64) -> usize {
    let mesh = s.triangulation(tol).to_polygon();
    mesh.faces().len()
}

#[no_mangle]
pub extern "C" fn smoke_run() -> u32 {
    let s = board_with_hole_and_dado();
    tri_count(&s, 0.1) as u32
}
