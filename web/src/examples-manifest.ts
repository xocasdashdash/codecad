export interface Example {
  id: string;
  title: string;
  desc: string;
}

export const EXAMPLES: Example[] = [
  {
    id: "01-flange-plate",
    title: "Flange Plate",
    desc: "DN80 PN16 plate flange: front view with bolt pattern and section A-A, ISO dimensions.",
  },
  {
    id: "02-fibonacci-spiral",
    title: "Fibonacci Spiral",
    desc: "Fibonacci square tiling with spiral and pole, plus the compass construction of the golden rectangle.",
  },
  {
    id: "03-roof-truss",
    title: "Howe Roof Truss",
    desc: "12 m timber truss with members at true depth, nail plates, bearings and chained dimensions.",
  },
  {
    id: "04-motor-schematic",
    title: "Motor Starter Schematic",
    desc: "IEC DOL starter: power and control circuits from a symbol block library, parts list.",
  },
  {
    id: "05-bicycle-frame",
    title: "Bicycle Frame",
    desc: "Road frame set out from stack, reach and angles; fused tube silhouette and geometry chart.",
  },
  {
    id: "06-piston-section",
    title: "Piston Cross-Section",
    desc: "Half-section piston and con-rod with a broken shank and a 3:1 detail of the ring grooves.",
  },
  {
    id: "07-floor-plan",
    title: "Floor Plan",
    desc: "Apartment plan: solid walls with cut openings, door blocks, room areas, chained dimensions.",
  },
  {
    id: "08-distribution-board",
    title: "Distribution Board",
    desc: "Single-line diagram: main switch, two RCD groups, ten MCB circuits, surge protection.",
  },
  {
    id: "09-spur-gears",
    title: "Spur Gears",
    desc: "Meshing involute gear pair at the true centre distance, line of action, gear data table.",
  },
  {
    id: "10-cam-follower",
    title: "Cam and Follower",
    desc: "Disc cam generated from a cycloidal motion law, roller follower and displacement diagram.",
  },
  {
    id: "11-pid",
    title: "P&ID",
    desc: "Feed tank and pump with level and flow control loops, ISA 5.1 instrument bubbles.",
  },
  {
    id: "12-site-plan",
    title: "Site Plan",
    desc: "Boundary from bearings and distances, setbacks, footprint, parking bays, spline contours.",
  },
];

/** Resolve DWG URL for an example. Base path depends on deployment. */
export function exampleDwgUrl(id: string): string {
  // On gh-pages: /examples/01-flange-plate.dwg
  // Dev / server: /examples/01-flange-plate.dwg
  return `${import.meta.env.BASE_URL}examples/${id}.dwg`;
}

/** Resolve preview image URL for an example. */
export function examplePngUrl(id: string): string {
  return `${import.meta.env.BASE_URL}examples/${id}.png`;
}
