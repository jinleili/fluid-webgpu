use crate::{PigmentParticle, PixelInfo, TrajectoryParticle};
use rand::Rng;

mod trajectory_render_node;
pub use trajectory_render_node::TrajectoryRenderNode;

pub fn init_trajectory_particles(num: wgpu::Extent3d, life_time: u32) -> Vec<TrajectoryParticle> {
    let mut data: Vec<TrajectoryParticle> = vec![];
    let mut rng = rand::thread_rng();
    let step_x = 2.0 / (num.width - 1) as f32;
    let step_y = 2.0 / (num.height - 1) as f32;
    for x in 0..num.width {
        let pixel_x = -1.0 + step_x * x as f32;
        for y in 0..num.height {
            let pos = [
                pixel_x + rng.gen_range(-step_x, step_x),
                -1.0 + step_y * y as f32 + rng.gen_range(-step_y, step_y),
            ];
            data.push(TrajectoryParticle {
                pos: pos,
                pos_initial: pos,
                life_time: rng.gen_range(0, life_time) as f32,
                fade: 1.0,
            });
        }
    }

    data
}

pub fn init_pigment_particles(num: u32, one_pixel_distance: f32) -> Vec<PigmentParticle> {
    // init position is at (0., -1)
    let mut data: Vec<PigmentParticle> = vec![];
    let mut rng = rand::thread_rng();
    let random_width = one_pixel_distance * 10.0;
    for x in 0..num {
        data.push(PigmentParticle {
            pos: [
                rng.gen_range(-random_width, random_width),
                -1.0 + rng.gen_range(0.0, random_width),
                0.0,
            ],
            diffuse: 0.0,
        });
    }

    data
}

pub fn init_canvas_data(sc_desc: &wgpu::SwapChainDescriptor) -> Vec<PixelInfo> {
    let mut data: Vec<PixelInfo> = vec![];
    for _ in 0..sc_desc.width {
        for _ in 0..sc_desc.height {
            data.push(PixelInfo { alpha: 0.0, speed: 0.0, rho: 0.0 });
        }
    }
    data
}
