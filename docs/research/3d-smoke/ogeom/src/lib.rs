use ogeom::core::Tolerances;
use ogeom::math::{Direction, Frame, Point, Vector};
use ogeom::mesh::Deflection;
use ogeom::topo::{explore_unique, Model, Shape, ShapeType};

pub const T: Tolerances = Tolerances::millimetres();

fn at(x: f64, y: f64, z: f64) -> Frame {
    Frame::new(Point::new(x, y, z), Direction::Z, Direction::X, T).unwrap()
}

/// Board with all 12 edges rounded (r=3), then a dowel hole and a dado cut.
pub fn board(model: &mut Model, round: bool) -> Shape {
    let mut b = ogeom::algo::make_box(model, Frame::WORLD, (600.0, 300.0, 18.0), T).unwrap().shape;
    if round {
        let edges = explore_unique(model, &b, ShapeType::Edge).unwrap();
        b = ogeom::fillet::fillet_edges(model, &b, &edges, 3.0, T).expect("fillet").shape;
    }
    let hole = ogeom::algo::make_cylinder(model, at(50.0, 50.0, -1.0), 5.0, 20.0, T).unwrap().shape;
    let dado = ogeom::algo::make_box(model, at(-1.0, 144.0, 12.0), (602.0, 12.0, 7.0), T).unwrap().shape;
    let s = ogeom::boolean::cut(model, &b, &hole, T).expect("cut hole").shape;
    ogeom::boolean::cut(model, &s, &dado, T).expect("cut dado").shape
}

pub fn tris(model: &Model, s: &Shape, chord: f64) -> usize {
    ogeom::mesh::triangulate(model, s, Deflection::with_chord(chord).unwrap(), T).unwrap().triangles.len()
}

pub fn hlr(model: &Model, s: &Shape) -> (usize, usize) {
    let view = ogeom::hlr::View::looking(-Vector::Y, Vector::Z, T).unwrap();
    let d = ogeom::hlr::exact::project_exact(model, s, &view, Deflection::default(), T).unwrap();
    (d.visible.len(), d.hidden.len())
}

#[no_mangle]
pub extern "C" fn smoke_run() -> u32 {
    let mut m = Model::new();
    let s = board(&mut m, true);
    tris(&m, &s, 0.1) as u32
}
