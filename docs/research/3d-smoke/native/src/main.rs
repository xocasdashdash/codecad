// Compile check: one wgpu (30) shared by eframe's egui-wgpu, vello and kiss3d,
// plus a wtransport client pinned to the server's self-signed cert hash.
fn _share(rs: &eframe::egui_wgpu::RenderState, adapter: wgpu_alias::Adapter, inst: wgpu_alias::Instance) -> vello::Renderer {
    // kiss3d takes the very same device/queue as its global context.
    kiss3d::context::Context::init(inst, rs.device.clone(), rs.queue.clone(), adapter, rs.target_format);
    vello::Renderer::new(&rs.device, vello::RendererOptions::default()).unwrap()
}
use vello::wgpu as wgpu_alias;
fn _client(hash: [u8; 32]) -> wtransport::ClientConfig {
    wtransport::ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes([wtransport::tls::Sha256Digest::new(hash)])
        .build()
}
fn main() { println!("ok"); }
