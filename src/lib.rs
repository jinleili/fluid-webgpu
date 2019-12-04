extern crate libc;
pub use idroid::utils::{depth_stencil, matrix_helper};
pub use uni_view::*;
use zerocopy::{AsBytes, FromBytes};

pub mod lbm;
pub mod optimized_mem_lbm;
pub mod particle;

// mod nse;
// pub use nse::Smoke2D;

mod ffi;
pub use ffi::*;

pub mod lattice;

#[derive(Copy, Clone)]
pub enum FlowType {
    Poiseuille,      // 泊萧叶流
    LidDrivenCavity, // 顶盖驱动方腔
    PigmentsDiffuse, // 颜料扩散
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
    pub lattice_num: [u32; 2],
    // 粒子数
    pub particle_num: [u32; 2],
    // 画布像素尺寸
    pub canvas_size: [u32; 2],
    // 正规化坐标空间里，一个像素对应的距离值
    pub pixel_distance: [f32; 2],
}

pub trait RenderNode {
    fn dispatch(&mut self, cpass: &mut wgpu::ComputePass);
    fn begin_render_pass(
        &mut self, device: &mut wgpu::Device, frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    );
}
