
pub trait RenderNode {
    fn dispatch(&mut self, cpass: &mut wgpu::ComputePass);
    fn begin_render_pass(
        &mut self, device: &mut wgpu::Device, frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    );
}
