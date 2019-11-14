#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FluidUniform {
    // 由于 OpenGL spec 定义的对齐方式，非 标量 或 vec2, 都是按 16 字节对齐
    // https://github.com/gfx-rs/wgpu-rs/issues/36
    e_and_w: [[f32; 4]; 9],
    // lattice 在正规化坐标空间的大小
    lattice_size: [f32; 2],
    // 格子数
    lattice_num: [f32; 2],
    particle_num: [f32; 2],
    pixel_distance: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ParticleUniform {
    // lattice 在正规化坐标空间的大小
    lattice_size: [f32; 2],
    lattice_num: [f32; 2],
    // 粒子数
    particle_num: [f32; 2],
    // 画布像素尺寸
    canvas_size: [f32; 2],
    // 正规化坐标空间里，一个像素对应的距离值
    pixel_distance: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Particle {
    pos: [f32; 2],
    pos_initial: [f32; 2],
    life_time: f32,
    fade: f32,
}

mod poiseuille_flow;
pub use poiseuille_flow::PoiseuilleFlow;

mod render_node;
use render_node::RenderNode;
