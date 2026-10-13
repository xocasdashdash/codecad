// Compile check: kiss3d and vello 0.11 in one crate graph must agree on one wgpu.
pub fn same_device(d: &kiss3d::context::Context) -> &vello::wgpu::Device { &d.device }
pub use kiss3d;
