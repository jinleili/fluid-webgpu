use idroid::node::ComputeNode;
use idroid::SurfaceView;
use wgpu::Extent3d;

use super::{ParticleUniform, RenderNode};
use uni_view::{AppView, GPUContext};

// 泊萧叶流
pub struct PoiseuilleFlow {
    app_view: AppView,
    lattice: Extent3d,

    boundary_node: ComputeNode,
    collide_node: ComputeNode,
    particle_node: RenderNode,

    swap: i32,
}

use super::FluidUniform;

fn init_data(nx: u32, ny: u32) -> (Vec<f32>, Vec<[f32; 4]>) {
    let mut lattice: Vec<f32> = vec![];
    let mut fluid: Vec<[f32; 4]> = vec![];
    for j in 0..ny {
        for i in 0..nx {
            for _ in 0..9 {
                lattice.push(0.0);
            }
            fluid.push([0.0, 0.0, 1.0, setup_open_geometry(i, j, nx, ny) as f32]);
        }
    }
    (lattice, fluid)
}

fn setup_open_geometry(x: u32, y: u32, nx: u32, ny: u32) -> u32 {
    // 不同的边用 10 的倍数来表示？
    // 上下两条边的墙
    // bounce back outer walls
    if y == 0 || y == ny - 1 {
        return 2;
    }
    if x == 0 {
        return 5; // inflow
    } else if x == nx - 1 {
        return 6; // outflow
    }

    // 障碍
    let half_size = 6;
    let size = half_size * 2;
    if (x > nx / 4
        && x < nx / 4 + size
        && y > ny / 2 - (half_size + 2)
        && y < ny / 2 + (half_size + 2))
        || (x > nx / 2 && x < nx / 2 + size && y > ny / 4 - half_size && y < ny / 4 + half_size)
        || (x > nx / 2
            && x < nx / 2 + size
            && y > (ny as f32 / 1.25) as u32 - half_size
            && y < (ny as f32 / 1.25) as u32 + half_size)
    {
        return 2;
    }

    return 1; // everything else shall be bulk fluid
}

fn get_fluid_uniform(
    lattice: Extent3d, particle: Extent3d, sc_desc: &wgpu::SwapChainDescriptor,
) -> FluidUniform {
    let w0 = 4.0 / 9.0;
    let w1 = 1.0 / 9.0;
    let w2 = 1.0 / 36.0;
    //  D2Q9 lattice :
    // 6 2 5
    // 3 0 1
    // 7 4 8
    // 按钮 屏幕 坐标取值的特点来指定方向坐标
    let e_and_w: [[f32; 4]; 9] = [
        [0.0, 0.0, w0, 0.0],
        [1.0, 0.0, w1, 0.0],
        [0.0, 1.0, w1, 0.0],
        [-1.0, 0.0, w1, 0.0],
        [0.0, -1.0, w1, 0.0],
        [1.0, 1.0, w2, 0.0],
        [-1.0, 1.0, w2, 0.0],
        [-1.0, -1.0, w2, 0.0],
        [1.0, -1.0, w2, 0.0],
    ];

    let uniform = FluidUniform {
        e_and_w,
        lattice_size: [2.0 / lattice.width as f32, 2.0 / lattice.height as f32],
        lattice_num: [lattice.width as f32, lattice.height as f32],
        particle_num: [particle.width as f32, particle.height as f32],
        pixel_distance: [2.0 / sc_desc.width as f32, 2.0 / sc_desc.height as f32],
    };
    uniform
}

impl PoiseuilleFlow {
    pub fn new(app_view: AppView) -> Self {
        let mut app_view = app_view;

        let lattice_num = (100, 75);
        let threadgroup_count: (u32, u32) = ((lattice_num.0 + 15) / 16, (lattice_num.1 + 15) / 16);

        let lattice = Extent3d { width: lattice_num.0, height: lattice_num.1, depth: 1 };
        let particle_num =
            Extent3d { width: lattice_num.0 * 1, height: lattice_num.1 * 1, depth: 1 };

        let swap = 0_i32;

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // lattice buffer bytes
        let buffer_range = (lattice.width * lattice.height * 9 * 4) as wgpu::BufferAddress;
        // macro fluid buffer bytes
        let fluid_buf_range = (lattice.width * lattice.height * 4 * 4) as wgpu::BufferAddress;

        let (lattice_data, fluid_data) = init_data(lattice.width, lattice.height);
        let (lattice0_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &lattice_data,
            buffer_range,
        );
        let (lattice1_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &lattice_data,
            buffer_range,
        );
        let (fluid_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &fluid_data,
            fluid_buf_range,
        );

        let uniform_size = std::mem::size_of::<FluidUniform>() as wgpu::BufferAddress;
        let uniform_buf = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            get_fluid_uniform(lattice, particle_num, &app_view.sc_desc),
            uniform_size,
        );

        let uniform_size2 = std::mem::size_of::<ParticleUniform>() as wgpu::BufferAddress;
        let uniform_buf2 = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            super::ParticleUniform {
                lattice_size: [2.0 / lattice.width as f32, 2.0 / lattice.height as f32],
                lattice_num: [lattice.width as f32, lattice.height as f32],
                particle_num: [particle_num.width as f32, particle_num.height as f32],
                canvas_size: [app_view.sc_desc.width as f32, app_view.sc_desc.height as f32],
                pixel_distance: [
                    2.0 / app_view.sc_desc.width as f32,
                    2.0 / app_view.sc_desc.height as f32,
                ],
            },
            uniform_size2,
        );

        // Create the render pipeline
        let boundary_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            &uniform_buf,
            uniform_size,
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer],
            vec![buffer_range, buffer_range, fluid_buf_range],
            vec![],
            ("lbm/poiseuille_boundary", env!("CARGO_MANIFEST_DIR")),
        );
        let collide_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            &uniform_buf,
            uniform_size,
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer],
            vec![buffer_range, buffer_range, fluid_buf_range],
            vec![],
            ("lbm/poiseuille_collide_streaming", env!("CARGO_MANIFEST_DIR")),
        );

        let particle_node = RenderNode::new(
            &app_view.sc_desc,
            &mut app_view.device,
            &mut encoder,
            vec![&uniform_buf, &uniform_buf2],
            vec![uniform_size, uniform_size2],
            &fluid_buffer,
            fluid_buf_range,
            particle_num,
        );

        let mut init_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            &uniform_buf,
            uniform_size,
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer],
            vec![buffer_range, buffer_range, fluid_buf_range],
            vec![],
            ("lbm/poiseuille_init", env!("CARGO_MANIFEST_DIR")),
        );
        init_node.compute(&mut app_view.device, &mut encoder);

        app_view.queue.submit(&[encoder.finish()]);

        PoiseuilleFlow { app_view, lattice, boundary_node, collide_node, particle_node, swap }
    }
}

impl SurfaceView for PoiseuilleFlow {
    fn scale(&mut self, _scale: f32) {}

    fn touch_moved(&mut self, _position: idroid::math::Position) {}

    fn resize(&mut self) {
        self.app_view.update_swap_chain();
    }

    fn enter_frame(&mut self) {
        self.swap += 1;
        // if self.swap % 10 != 0 {
        //     return;
        // }

        let mut encoder = self
            .app_view
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.boundary_node.compute(&mut self.app_view.device, &mut encoder);
        self.collide_node.compute(&mut self.app_view.device, &mut encoder);

        let frame = self
            .app_view
            .swap_chain
            .get_next_texture()
            .expect("swap_chain.get_next_texture() timeout");
        {
            self.particle_node.begin_render_pass(&mut self.app_view.device, &frame, &mut encoder);
        }

        self.app_view.queue.submit(&[encoder.finish()]);
        // println!("{:?}", (self.swap / 10) % 60);
    }
}
