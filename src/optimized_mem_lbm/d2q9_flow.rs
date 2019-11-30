use idroid::node::ComputeNode;
use idroid::SurfaceView;
use wgpu::Extent3d;

use super::init_data::{get_fluid_uniform, init_data};
use super::CollideStreamNode;

use crate::{D2Q9Uniform, FlowType, FluidUniform2, RenderNode, TrajectoryRenderNode};
use uni_view::{AppView, GPUContext};

pub struct D2Q9Flow {
    app_view: AppView,
    flow_type: FlowType,
    lattice: Extent3d,

    collide_stream_node: CollideStreamNode,
    boundary_node: ComputeNode,
    macro_node: ComputeNode,
    particle_node: Box<dyn RenderNode>,

    swap: i32,
}

impl D2Q9Flow {
    pub fn new(app_view: AppView, flow_type: FlowType) -> Self {
        let mut app_view = app_view;

        let (lattice_num, particle_num) = match flow_type {
            FlowType::poiseuille => ((100, 75), Extent3d { width: 100, height: 75, depth: 1 }),
            // FlowType::poiseuille => ((55, 55), Extent3d { width: 55, height: 55, depth: 1 }),
            FlowType::lid_driven_cavity => {
                ((100, 100), Extent3d { width: 75, height: 50, depth: 1 })
            }
            FlowType::pigments_diffuse => ((200, 150), Extent3d { width: 0, height: 0, depth: 0 }),
        };
        let threadgroup_count: (u32, u32) = ((lattice_num.0 + 15) / 16, (lattice_num.1 + 15) / 16);
        let lattice = Extent3d { width: lattice_num.0, height: lattice_num.1, depth: 1 };

        let swap = 0_i32;

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // lattice buffer bytes
        let buffer_range = (lattice.width * lattice.height * 9 * 4) as wgpu::BufferAddress;
        let temp_buffer_range = (lattice.width * lattice.height * 4) as wgpu::BufferAddress;

        // macro fluid buffer bytes
        let fluid_buf_range = (lattice.width * lattice.height * 4 * 4) as wgpu::BufferAddress;
        let temp_fluid_range = (lattice.width * lattice.height * 4 * 3) as wgpu::BufferAddress;

        let (lattice_data, temp_scalar_data, fluid_data, temp_fluid_data) =
            init_data(lattice.width, lattice.height, flow_type);
        let (lattice_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &lattice_data,
            buffer_range,
        );
        let (temp_scalar_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &temp_scalar_data,
            temp_buffer_range,
        );

        let (fluid_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &fluid_data,
            fluid_buf_range,
        );

        let (temp_fluid_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &temp_fluid_data,
            temp_fluid_range,
        );

        let (d2q9_uniform_data, fluid_uniform_data) =
            get_fluid_uniform(lattice, particle_num, flow_type, &app_view.sc_desc);
        let uniform_size0 = std::mem::size_of::<D2Q9Uniform>() as wgpu::BufferAddress;
        let uniform_buf0 = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            d2q9_uniform_data,
            uniform_size0,
        );

        let uniform_size = std::mem::size_of::<FluidUniform2>() as wgpu::BufferAddress;
        let uniform_buf = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            fluid_uniform_data,
            uniform_size,
        );
        let collide_stream_node = CollideStreamNode::new(
            &mut app_view.device,
            lattice,
            vec![&uniform_buf0, &uniform_buf],
            vec![uniform_size0, uniform_size],
            vec![&lattice_buffer, &temp_scalar_buffer, &fluid_buffer, &temp_fluid_buffer],
            vec![buffer_range, temp_buffer_range, fluid_buf_range, temp_fluid_range],
            vec![],
        );

        let boundary_shader = match flow_type {
            FlowType::poiseuille | FlowType::pigments_diffuse => "optimized_mem_lbm/boundary",
            FlowType::lid_driven_cavity => "optimized_mem_lbm/lid_driven_boundary",
        };
        let boundary_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![uniform_size0, uniform_size],
            vec![&lattice_buffer, &temp_scalar_buffer, &fluid_buffer, &temp_fluid_buffer],
            vec![buffer_range, temp_buffer_range, fluid_buf_range, temp_fluid_range],
            vec![],
            (boundary_shader, env!("CARGO_MANIFEST_DIR")),
        );

        let macro_shader = match flow_type {
            FlowType::poiseuille | FlowType::pigments_diffuse => "optimized_mem_lbm/macro",
            FlowType::lid_driven_cavity => "optimized_mem_lbm/macro",
        };
        let macro_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![uniform_size0, uniform_size],
            vec![&lattice_buffer, &temp_scalar_buffer, &fluid_buffer, &temp_fluid_buffer],
            vec![buffer_range, temp_buffer_range, fluid_buf_range, temp_fluid_range],
            vec![],
            (macro_shader, env!("CARGO_MANIFEST_DIR")),
        );

        let particle_node: Box<dyn RenderNode> = Box::new(TrajectoryRenderNode::new(
            &app_view.sc_desc,
            &mut app_view.device,
            &mut encoder,
            &fluid_buffer,
            fluid_buf_range,
            flow_type,
            lattice,
            particle_num,
        ));

        let mut init_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![uniform_size0, uniform_size],
            vec![&lattice_buffer, &temp_scalar_buffer, &fluid_buffer, &temp_fluid_buffer],
            vec![buffer_range, temp_buffer_range, fluid_buf_range, temp_fluid_range],
            vec![],
            ("optimized_mem_lbm/init", env!("CARGO_MANIFEST_DIR")),
        );
        init_node.compute(&mut app_view.device, &mut encoder);

        app_view.queue.submit(&[encoder.finish()]);

        D2Q9Flow {
            app_view,
            flow_type,
            lattice,
            boundary_node,
            collide_stream_node,
            macro_node,
            particle_node,
            swap,
        }
    }
}

impl SurfaceView for D2Q9Flow {
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
        {
            let mut cpass = encoder.begin_compute_pass();
            self.collide_stream_node.dispatch(&mut cpass);
            self.boundary_node.dispatch(&mut cpass);
            self.macro_node.dispatch(&mut cpass);

            self.particle_node.dispatch(&mut cpass);
        }

        let frame = self
            .app_view
            .swap_chain
            .get_next_texture()
            .expect("swap_chain.get_next_texture() timeout");
        {
            self.particle_node.begin_render_pass(&mut self.app_view.device, &frame, &mut encoder);
        }

        self.app_view.queue.submit(&[encoder.finish()]);
        // println!("{:?}", (self.swap) % 600);
    }
}
