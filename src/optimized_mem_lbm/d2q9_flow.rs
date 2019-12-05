use idroid::buffer::*;
use idroid::node::ComputeNode;
use idroid::SurfaceView;
use wgpu::Extent3d;

use super::CollideStreamNode;
use crate::lattice::{fluid_uniform, setup_lattice, LatticeInfo, MacroInfo};

use crate::particle::{PigmentDiffuseRenderNode, RenderNode, TrajectoryRenderNode};
use crate::{D2Q9Uniform, FlowType, FluidUniform};
use uni_view::{AppView, GPUContext};

pub struct D2Q9Flow {
    app_view: AppView,
    flow_type: FlowType,
    collide_stream_node: CollideStreamNode,
    boundary_node: ComputeNode,
    particle_node: Box<dyn RenderNode>,
    diffuse_collide_stream_node: Option<CollideStreamNode>,
    diffuse_boundary_node: Option<ComputeNode>,
    swap: i32,
}

impl D2Q9Flow {
    pub fn new(app_view: AppView, flow_type: FlowType) -> Self {
        let mut app_view = app_view;

        let (lattice_num, particle_num) = match flow_type {
            FlowType::Poiseuille => ((200, 150), Extent3d { width: 200, height: 150, depth: 1 }),
            // FlowType::Poiseuille => ((64, 48), Extent3d { width: 64, height: 48, depth: 1 }),
            FlowType::LidDrivenCavity => ((100, 100), Extent3d { width: 75, height: 50, depth: 1 }),
            FlowType::PigmentsDiffuse => {
                if cfg!(target_os = "macos") {
                    ((200, 150), Extent3d { width: 0, height: 0, depth: 0 })
                } else {
                    ((150, 250), Extent3d { width: 0, height: 0, depth: 0 })
                }
            }
            _ => panic!("not implemented flow_type"),
        };
        let threadgroup_count: (u32, u32) = ((lattice_num.0 + 15) / 16, (lattice_num.1 + 15) / 16);
        let lattice = Extent3d { width: lattice_num.0, height: lattice_num.1, depth: 1 };

        let swap = 0_i32;

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let (lattice_info_data, lattice_data, temp_scalar_data, macro_data) =
            init_data(lattice.width, lattice.height, flow_type);

        let lattice_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &lattice_data,
        );
        let info_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &lattice_info_data,
        );
        let temp_scalar_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &temp_scalar_data,
        );
        let macro_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &macro_data,
        );
        let diffuse_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &lattice_data,
        );
        let diffuse_scalar_buffer = BufferObj::create_storage_buffer(
            &mut app_view.device,
            &temp_scalar_data,
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
        let base_buffers: Vec<&BufferObj> =
            vec![&lattice_buffer, &temp_scalar_buffer, &macro_buffer, &info_buffer];

        let boundary_shader = match flow_type {
            FlowType::Poiseuille | FlowType::PigmentsDiffuse => "optimized_mem_lbm/boundary",
            FlowType::LidDrivenCavity => "optimized_mem_lbm/lid_driven_boundary",
            _ => panic!("not implemented flow_type"),
        };
        let boundary_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            base_buffers.clone(),
            vec![],
            (boundary_shader, env!("CARGO_MANIFEST_DIR")),
        );

        let particle_node: Box<dyn RenderNode>;

        let mut diffuse_collide_stream_node: Option<CollideStreamNode> = None;
        let mut diffuse_boundary_node: Option<ComputeNode> = None;
        let mut init_buffers: Vec<&BufferObj> = base_buffers.clone();
        let init_shader: &str;
        let collide_shader: &str;

        match flow_type {
            FlowType::PigmentsDiffuse => {
                diffuse_collide_stream_node = Some(CollideStreamNode::new(
                    &mut app_view.device,
                    lattice,
                    vec![&uniform_buf0, &uniform_buf],
                    {
                        let mut buffers = base_buffers.clone();
                        buffers.append(&mut vec![&diffuse_buffer, &diffuse_scalar_buffer]);
                        buffers
                    },
                    "optimized_mem_lbm/diffuse/collide",
                    "optimized_mem_lbm/diffuse/stream",
                ));
                diffuse_boundary_node = Some(ComputeNode::new(
                    &mut app_view.device,
                    threadgroup_count,
                    vec![&uniform_buf0, &uniform_buf],
                    {
                        let mut buffers = base_buffers.clone();
                        buffers.push(&diffuse_buffer);
                        buffers
                    },
                    vec![],
                    ("optimized_mem_lbm/diffuse/boundary", env!("CARGO_MANIFEST_DIR")),
                ));
                particle_node = Box::new(PigmentDiffuseRenderNode::new(
                    &app_view.sc_desc,
                    &mut app_view.device,
                    &macro_buffer,
                    &diffuse_scalar_buffer,
                    flow_type,
                    lattice,
                    particle_num,
                ));
                collide_shader = "optimized_mem_lbm/diffuse/advect_collide";
                init_buffers.push(&diffuse_buffer);
                init_shader = "optimized_mem_lbm/diffuse/init";
            }
            _ => {
                particle_node = Box::new(TrajectoryRenderNode::new(
                    &app_view.sc_desc,
                    &mut app_view.device,
                    &macro_buffer,
                    &info_buffer,
                    flow_type,
                    lattice,
                    particle_num,
                ));
                collide_shader = "optimized_mem_lbm/collide";
                init_shader = "optimized_mem_lbm/init";
            }
        };
        let collide_stream_node = CollideStreamNode::new(
            &mut app_view.device,
            lattice,
            vec![&uniform_buf0, &uniform_buf],
            base_buffers.clone(),
            collide_shader,
            "optimized_mem_lbm/stream",
        );
        let mut init_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf],
            init_buffers,
            vec![],
            (init_shader, env!("CARGO_MANIFEST_DIR")),
        );
        init_node.compute(&mut app_view.device, &mut encoder);

        app_view.queue.submit(&[encoder.finish()]);

        D2Q9Flow {
            app_view,
            flow_type,
            boundary_node,
            collide_stream_node,
            diffuse_collide_stream_node,
            diffuse_boundary_node,
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
        let mut encoder = self
            .app_view
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut cpass = encoder.begin_compute_pass();
            self.collide_stream_node.dispatch(&mut cpass);
            self.boundary_node.dispatch(&mut cpass);
            if let FlowType::PigmentsDiffuse = self.flow_type {
                if let Some(diffuse_collide_stream_node) = &mut self.diffuse_collide_stream_node {
                    diffuse_collide_stream_node.dispatch(&mut cpass);
                }
                if let Some(diffuse_boundary_node) = &mut self.diffuse_boundary_node {
                    diffuse_boundary_node.dispatch(&mut cpass);
                }
            }
            self.particle_node.dispatch(&mut cpass);
        }

        let frame = self
            .app_view
            .swap_chain
            .get_next_texture()
            .expect("swap_chain.get_next_texture() timeout");
        self.particle_node.begin_render_pass(&mut self.app_view.device, &frame, &mut encoder);

        self.app_view.queue.submit(&[encoder.finish()]);
    }
}

pub fn init_data(
    nx: u32, ny: u32, flow_type: FlowType,
) -> (Vec<LatticeInfo>, Vec<f32>, Vec<f32>, Vec<MacroInfo>) {
    let mut lattice: Vec<f32> = vec![];
    let mut temp_scalar_lattice: Vec<f32> = vec![];
    let mut fluid: Vec<MacroInfo> = vec![];
    let mut info: Vec<LatticeInfo> = vec![];

    for j in 0..ny {
        for i in 0..nx {
            for _ in 0..9 {
                lattice.push(0.0);
            }
            temp_scalar_lattice.push(0.0);
            fluid.push(MacroInfo { velocity: [0.0, 0.0], rho: 1.0, any: 0.0 });
            info.push(LatticeInfo {
                material: setup_lattice(i, j, nx, ny, flow_type) as i32,
                iter: 0.0,
                threshold: 0.0,
            })
        }
    }
    (info, lattice, temp_scalar_lattice, fluid)
}
