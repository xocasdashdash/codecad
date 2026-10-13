# 3D smoke tests

These are the small standalone crates behind the evidence in
[../../3d-design.md](../../3d-design.md) §2. Each one is its own cargo
workspace (it has its own `[workspace]` table), so none of them is part of
the CodeCAD workspace and none touches `target/`. Only sources are
committed: no lockfiles, no build output.

All modelling tests build the same part: a 600 x 300 x 18 mm board, minus a
Ø10 through hole, minus a 12 x 6 dado along its length. The closed-form
volume is 3 195 386.283 mm³. With all 12 edges rounded at r = 3 before the
cuts it is 3 188 376.6 mm³.

| Crate | What it checks |
|---|---|
| `ogeom/` | ogeom 0.10: booleans, the 12-edge fillet, validity, exact volume, tessellation at three tolerances, front-view HLR. The lib exports `smoke_run` for wasm. |
| `truck/` | truck-modeling 0.6 / truck-shapeops 0.4: booleans, then mesh volume at decreasing tolerance. The lib exports `smoke_run`. |
| `vcad/` | vcad-kernel 0.10: booleans with and without a fillet, and the result's `fidelity()`. |
| `csgrs/` | csgrs main (pinned git rev): box minus box, and box minus cylinders with 8/16/32/64 segments. (The crates.io 0.20.1 release no longer resolves: it depends on a yanked `core2`.) |
| `kiss/` | Compile check: kiss3d 0.47 and vello 0.11 resolve to one wgpu. |
| `rl/` | Compile check: renderling main (pinned git rev) for wasm32. It uses wgpu 26. |
| `native/` | Compile check: eframe 0.36.2 (wgpu), egui_dock, vello 0.11, kiss3d 0.47 and a wtransport 0.7.2 client in one binary, sharing one device. |
| `runwasm.js` | Runs an exported `smoke_run` from a `wasm32-unknown-unknown` build in Node, with every import stubbed to throw. |

## Running

Use a separate target directory outside the repo:

```bash
export CARGO_TARGET_DIR=$HOME/.cache/codecad-3d-smoke   # any path outside the repo

# Native run (ogeom, truck, vcad, csgrs have a binary)
(cd docs/research/3d-smoke/ogeom && cargo run --release)

# wasm build + run in Node (ogeom, truck, vcad)
(cd docs/research/3d-smoke/ogeom && cargo build --release --lib --target wasm32-unknown-unknown)
node docs/research/3d-smoke/runwasm.js "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/smoke_ogeom.wasm" smoke_run

# Compile checks
(cd docs/research/3d-smoke/kiss && cargo build --release --target wasm32-unknown-unknown)
(cd docs/research/3d-smoke/rl && cargo build --release --target wasm32-unknown-unknown)
(cd docs/research/3d-smoke/native && cargo build --release)
cargo tree --manifest-path docs/research/3d-smoke/native/Cargo.toml -d   # expect no duplicate wgpu/winit/vello
```

## Results recorded on 2026-10-13

These were measured with rustc 1.98, the `opt-level = "s"` + LTO release
profile, and no GPU on the machine. An independent rerun reproduced the
ogeom numbers exactly.

| Test | Result |
|---|---|
| ogeom, plain board | 5–12 ms, 11 faces, valid, volume **3 195 386.283** |
| ogeom, rounded board | 0.6–1.1 s, 33 faces, valid, volume **3 188 376.635**. HLR front view: 55 visible / 46 hidden curves in 9 ms |
| ogeom, wasm | 3.2 MB (1.08 MB gzip), **no imports**, rounded board in 3.3 s in Node |
| truck | 2 booleans in 344–389 ms, 13 faces. Mesh volume 3 195 386.538 at tol 0.001 |
| truck, wasm | 949 KB (260 KB gzip), runs in 1.37 s (wasm-bindgen placeholder imports, never called) |
| vcad | plain: `Analytic`, volume 3 195 386.66 (mesh). Rounded: **`TriangleSoup`** (booleans fell back to mesh CSG). wasm 1.3 MB, 1.2 s |
| csgrs main | box minus box ok, box minus 8-segment cylinder ok, **16/32/64 segments fail**: `predicate could not be decided: Real sign classification` |
| kiss + vello | compiles for wasm32 with a single `wgpu v30.0.1` |
| renderling main | compiles for wasm32, on `wgpu v26` |
| native | compiles, no duplicate wgpu, winit or vello |

Truck's unreleased master fillet tests (`cargo test --release -p truck-shapeops fillet` in a truck checkout at `88ed005`) passed: `create_simple_fillet` and `create_fillet_with_side`.
