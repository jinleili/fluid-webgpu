use crate::{D2Q9Uniform, FlowType, FluidUniform};
use wgpu::Extent3d;
use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct LatticeInfo {
    pub material: f32,
    pub diffuse_step_count: f32,
    //  dynamic iter value, change material ultimately
    pub iter: f32,
    pub threshold: f32,
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct MacroInfo {
    pub velocity: [f32; 2],
    pub rho: f32,
    pub any: f32,
}

pub fn setup_lattice(x: u32, y: u32, nx: u32, ny: u32, flow_type: FlowType) -> u32 {
    // 不同的边用 10 的倍数来表示？
    match flow_type {
        FlowType::Poiseuille => {
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
            if (x > nx / 4 && x < nx / 4 + size && y > ny / 2 - (half_size + 2) && y < ny / 2 + (half_size + 2))
                || (x > nx / 2 && x < nx / 2 + size && y > ny / 4 - half_size && y < ny / 4 + half_size)
                || (x > nx / 2
                    && x < nx / 2 + size
                    && y > (ny as f32 / 1.25) as u32 - half_size
                    && y < (ny as f32 / 1.25) as u32 + half_size)
            {
                return 2;
            }
        }
        FlowType::LidDrivenCavity => {
            if y == 0 && (x > 0 && x < nx - 1) {
                return 3; // lid-driven wall
            } else if x == 0 || x == nx - 1 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
        }
        FlowType::PigmentsDiffuse => {
            if y == 0 && x > nx / 2 - 10 && x < nx / 2 + 10 {
                return 5; // inflow
            } else if x == 0 || x == nx - 1 || y == 0 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
        }
        _ => {
            if x == 0 || x == nx - 1 || y == 0 || y == ny - 1 {
                return 2; // bounce back outer walls
            }
        }
    }

    return 1; // everything else shall be bulk fluid
}

pub fn fluid_uniform(
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
        FlowType::Poiseuille | FlowType::PigmentsDiffuse => 0.83,
        FlowType::LidDrivenCavity => {
            // lid-driven cavity flow's parameter: viscosity 0.01, lattice 100*100, U = 0.1
            0.5 * (1.0 + 6.0 * 0.01)
        }
        _ => 0.83,
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
