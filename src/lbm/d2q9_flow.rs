use idroid::buffer::*;
use idroid::node::ComputeNode;
use idroid::SurfaceView;

use wgpu::Extent3d;

use crate::lattice::{fluid_uniform, setup_lattice, LatticeInfo, MacroInfo};
use crate::particle::{RenderNode, TrajectoryRenderNode};
use crate::{FlowType};
use uni_view::{AppView, GPUContext};

pub struct D2Q9Flow {
    app_view: AppView,
    stream_node: ComputeNode,
    collide_node: ComputeNode,
    particle_node: Box<dyn RenderNode>,
    swap: i32,
}

impl D2Q9Flow {
    pub fn new(app_view: AppView, flow_type: FlowType) -> Self {
        let mut app_view = app_view;

        let (lattice_num, particle_num) = match flow_type {
            FlowType::Poiseuille => ((200, 150), Extent3d { width: 100, height: 75, depth: 1 }),
            FlowType::LidDrivenCavity => ((100, 100), Extent3d { width: 75, height: 50, depth: 1 }),
            FlowType::PigmentsDiffuse => ((200, 150), Extent3d { width: 0, height: 0, depth: 0 }),
        };
        let threadgroup_count: (u32, u32) = ((lattice_num.0 + 15) / 16, (lattice_num.1 + 15) / 16);
        let lattice = Extent3d { width: lattice_num.0, height: lattice_num.1, depth: 1 };

        let swap = 0_i32;

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let (lattice_info_data, lattice_data, fluid_data) =
            init_data(lattice.width, lattice.height, flow_type);

        let lattice0_buffer = BufferObj::create_storage_buffer(&mut app_view.device, &lattice_data);
        let lattice1_buffer = BufferObj::create_storage_buffer(&mut app_view.device, &lattice_data);

        let info_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &lattice_info_data,
        );
        let fluid_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &fluid_data,
        );

        let (d2q9_uniform_data, fluid_uniform_data) =
            fluid_uniform(lattice, particle_num, flow_type, &app_view.sc_desc);
        let uniform_buf0 = BufferObj::create_uniform_buffer(
            &mut app_view.device,
            &d2q9_uniform_data,
        );

        let uniform_buf = BufferObj::create_uniform_buffer(
            &mut app_view.device,
            &fluid_uniform_data,
        );

        // Create the render pipeline
        let stream_shader = match flow_type {
            FlowType::Poiseuille | FlowType::PigmentsDiffuse => "lbm/poiseuille_stream",
            FlowType::LidDrivenCavity => "lbm/lid_driven_stream",
        };
        let stream_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer, &info_buffer],
            vec![],
            (stream_shader, env!("CARGO_MANIFEST_DIR")),
        );
        let collide_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer, &info_buffer],
            vec![],
            ("lbm/d2q9_collide", env!("CARGO_MANIFEST_DIR")),
        );

        let particle_node: Box<dyn RenderNode> = match flow_type {
            FlowType::Poiseuille | FlowType::LidDrivenCavity => {
                Box::new(TrajectoryRenderNode::new(
                    &app_view.sc_desc,
                    &mut app_view.device,
                    &fluid_buffer,
                    &info_buffer,
                    flow_type,
                    lattice,
                    particle_num,
                ))
            }
            FlowType::PigmentsDiffuse => panic!("pigments_diffuse not implemented!"),
        };

        let mut init_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            vec![&lattice0_buffer, &lattice1_buffer, &fluid_buffer, &info_buffer],
            vec![],
            ("lbm/d2q9_init", env!("CARGO_MANIFEST_DIR")),
        );
        init_node.compute(&mut app_view.device, &mut encoder);

        app_view.queue.submit(&[encoder.finish()]);

        D2Q9Flow { app_view, stream_node, collide_node, particle_node, swap }
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
            self.stream_node.dispatch(&mut cpass);
            self.collide_node.dispatch(&mut cpass);
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

pub fn init_data(
    nx: u32, ny: u32, flow_type: FlowType,
) -> (Vec<LatticeInfo>, Vec<f32>, Vec<MacroInfo>) {
    let mut lattice: Vec<f32> = vec![];
    let mut fluid: Vec<MacroInfo> = vec![];
    let mut info: Vec<LatticeInfo> = vec![];

    for j in 0..ny {
        for i in 0..nx {
            for _ in 0..9 {
                lattice.push(0.0);
            }
            fluid.push(MacroInfo { velocity: [0.0, 0.0], rho: 1.0, any: 0.0 });
            info.push(LatticeInfo {
                material: setup_lattice(i, j, nx, ny, flow_type) as i32,
                iter: 0.0,
                threshold: 0.0,
            })
        }
    }
    (info, lattice, fluid)
}
