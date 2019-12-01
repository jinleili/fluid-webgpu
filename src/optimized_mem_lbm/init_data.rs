use crate::{D2Q9Uniform, FlowType, FluidUniform, PigmentParticle, PixelInfo};
use rand::Rng;
use wgpu::Extent3d;

pub fn init_data(
    nx: u32, ny: u32, flow_type: FlowType,
) -> (Vec<f32>, Vec<f32>, Vec<[f32; 4]>, Vec<[f32; 3]>) {
    let mut lattice: Vec<f32> = vec![];
    let mut temp_scalar_lattice: Vec<f32> = vec![];
    let mut fluid: Vec<[f32; 4]> = vec![];
    let mut temp_fluid: Vec<[f32; 3]> = vec![];

    for j in 0..ny {
        for i in 0..nx {
            for _ in 0..9 {
                lattice.push(0.0);
            }
            temp_scalar_lattice.push(0.0);

            fluid.push([0.0, 0.0, 1.0, setup_open_geometry(i, j, nx, ny, flow_type) as f32]);
            temp_fluid.push([0.0, 0.0, 0.0]);
        }
    }
    (lattice, temp_scalar_lattice, fluid, temp_fluid)
}

fn setup_open_geometry(x: u32, y: u32, nx: u32, ny: u32, flow_type: FlowType) -> u32 {
    // less than 10: boundary or obstacle
    // 1: up, 2: right, 3: bottom, 4: left
    match flow_type {
        FlowType::poiseuille => {
            if y == 0 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
            if x == 0 {
                return 5; // inflow
            } else if x == nx - 1 {
                return 6; // outflow
            }

            // obstacle
            let half_size = 6;
            let size = half_size * 2;
            if (x > nx / 4
                && x < nx / 4 + size
                && y > ny / 2 - (half_size + 2)
                && y < ny / 2 + (half_size + 2))
                || (x > nx / 2
                    && x < nx / 2 + size
                    && y > ny / 4 - half_size
                    && y < ny / 4 + half_size)
                || (x > nx / 2
                    && x < nx / 2 + size
                    && y > (ny as f32 / 1.25) as u32 - half_size
                    && y < (ny as f32 / 1.25) as u32 + half_size)
            {
                return 2;
            }
        }
        FlowType::lid_driven_cavity => {
            if y == 0 && (x > 0 && x < nx - 1) {
                return 3; // lid-driven wall
            } else if x == 0 || x == nx - 1 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
        }
        FlowType::pigments_diffuse => {
            if x == 0 || x == nx - 1 || y == 0 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
        }
    }

    return 1; // everything else shall be bulk fluid
}

pub fn get_fluid_uniform(
    lattice: Extent3d, particle: Extent3d, flow_type: FlowType, sc_desc: &wgpu::SwapChainDescriptor,
) -> (D2Q9Uniform, FluidUniform) {
    let w0 = 4.0 / 9.0;
    let w1 = 1.0 / 9.0;
    let w2 = 1.0 / 36.0;
    //  D2Q9 lattice :
    // 6 2 5
    // 3 0 1
    // 7 4 8
    // 按钮 屏幕 坐标取值的特点来指定方向坐标
    // let e_and_w: [[f32; 4]; 9] = [
    //     [0.0, 0.0, w0, 0.0],
    //     [1.0, 0.0, w1, 0.0],
    //     [0.0, 1.0, w1, 0.0],
    //     [-1.0, 0.0, w1, 0.0],
    //     [0.0, -1.0, w1, 0.0],
    //     [1.0, 1.0, w2, 0.0],
    //     [-1.0, 1.0, w2, 0.0],
    //     [-1.0, -1.0, w2, 0.0],
    //     [1.0, -1.0, w2, 0.0],
    // ];
    let e_and_w: [[f32; 4]; 9] = [
        [0.0, 0.0, w0, 0.0],
        [1.0, 0.0, w1, 0.0],
        [0.0, -1.0, w1, 0.0],
        [-1.0, 0.0, w1, 0.0],
        [0.0, 1.0, w1, 0.0],
        [1.0, -1.0, w2, 0.0],
        [-1.0, -1.0, w2, 0.0],
        [-1.0, 1.0, w2, 0.0],
        [1.0, 1.0, w2, 0.0],
    ];

    let tau = match flow_type {
        FlowType::poiseuille | FlowType::pigments_diffuse => 0.83,
        FlowType::lid_driven_cavity => {
            // lid-driven cavity flow's parameter: viscosity 0.01, lattice 100*100, U = 0.1
            0.5 * (1.0 + 6.0 * 0.01)
        }
    };

    let uniform = FluidUniform {
        lattice_size: [2.0 / lattice.width as f32, 2.0 / lattice.height as f32],
        lattice_num: [lattice.width, lattice.height],
        particle_num: [particle.width, particle.height],
        pixel_distance: [2.0 / sc_desc.width as f32, 2.0 / sc_desc.height as f32],
        tau_and_omega: [tau, 1.0 / tau],
    };

    (D2Q9Uniform { e_and_w }, uniform)
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
