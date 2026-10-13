use smoke_ogeom::*;
fn main() {
    for round in [false, true] {
        let mut m = ogeom::topo::Model::new();
        let t = std::time::Instant::now();
        let s = board(&mut m, round);
        let dt = t.elapsed();
        let faces = ogeom::topo::explore_unique(&m, &s, ogeom::topo::ShapeType::Face).unwrap().len();
        let valid = ogeom::algo::check(&m, &s, T).unwrap().is_valid();
        println!("round={round}: build {dt:?}, {faces} faces, valid={valid}");
        for c in [1.0, 0.1, 0.01] { let t=std::time::Instant::now(); println!("  chord {c}: {} tris ({:?})", tris(&m, &s, c), t.elapsed()); }
        let t = std::time::Instant::now();
        let (v, h) = hlr(&m, &s);
        let vol = ogeom::algo::volume_properties(&m, &s, ogeom::mesh::Deflection::default(), T).unwrap().mass; println!("  volume {vol:.3} (exact expected unrounded 3195386.282)");
        println!("  front-view HLR: {v} visible, {h} hidden curves ({:?})", t.elapsed());
    }
}
