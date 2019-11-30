extern crate libc;
pub use idroid::utils::{depth_stencil, matrix_helper};
pub use uni_view::*;
use zerocopy::{AsBytes, FromBytes};

pub mod lbm;
pub mod optimized_mem_lbm;

mod nse;
pub use nse::Smoke2D;

mod particle;
pub use particle::TrajectoryRenderNode;


#[derive(Copy, Clone)]
pub enum FlowType {
    poiseuille,        // 泊萧叶流
    lid_driven_cavity, // 顶盖驱动方腔
    pigments_diffuse,  // 颜料扩散
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct D2Q9Uniform {
    // 由于 OpenGL spec 定义的对齐方式，非 标量 或 vec2, 都是按 16 字节对齐
    // https://github.com/gfx-rs/wgpu-rs/issues/36
    pub e_and_w: [[f32; 4]; 9],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct FluidUniform {
    // lattice 在正规化坐标空间的大小
    pub lattice_size: [f32; 2],
    // 格子数
    pub lattice_num: [f32; 2],
    pub particle_num: [f32; 2],
    pub pixel_distance: [f32; 2],
    pub tau_and_omega: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct FluidUniform2 {
    // lattice 在正规化坐标空间的大小
    pub lattice_size: [f32; 2],
    // 格子数
    pub lattice_num: [u32; 2],
    pub particle_num: [u32; 2],
    pub pixel_distance: [f32; 2],
    pub tau_and_omega: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct ParticleUniform {
    // lattice 在正规化坐标空间的大小
    pub lattice_size: [f32; 2],
    pub lattice_num: [f32; 2],
    // 粒子数
    pub particle_num: [f32; 2],
    // 画布像素尺寸
    pub canvas_size: [f32; 2],
    // 正规化坐标空间里，一个像素对应的距离值
    pub pixel_distance: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct AnimateUniform {
    pub life_time: f32,
    pub fade_out_factor: f32,
    pub speed_factor: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct TrajectoryParticle {
    pub pos: [f32; 2],
    pub pos_initial: [f32; 2],
    pub life_time: f32,
    pub fade: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct PigmentParticle {
    pub pos: [f32; 3],
    pub diffuse: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct PixelInfo {
    pub alpha: f32,
    // absolute velocity
    pub speed: f32,
    // density
    pub rho: f32,
}

pub trait RenderNode {
    fn dispatch(&mut self, cpass: &mut wgpu::ComputePass);
    fn begin_render_pass(
        &mut self, device: &mut wgpu::Device, frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    );
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_poiseuille_view(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, lbm::FlowType::poiseuille);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_lip_driven_cavity(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, lbm::FlowType::lid_driven_cavity);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_pigments_diffuse(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, lbm::FlowType::pigments_diffuse);
    idroid::box_obj(obj)
}
