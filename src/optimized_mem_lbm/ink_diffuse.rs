use idroid::buffer::*;
use idroid::node::ComputeNode;
use idroid::SurfaceView;
use wgpu::{Extent3d, TextureView};

use super::CollideStreamNode;
use crate::lattice::{fluid_uniform, setup_lattice, LatticeInfo};

use crate::particle::{PigmentDiffuseRenderNode, RenderNode};
use crate::FlowType;
use uni_view::{fs::FileSystem, AppView, GPUContext};

use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes)]
pub struct TouchUniform {
    pub touch_point: [i32; 2],
    // left top lattice index, maybe is a negtive number
    pub lt_lattice: [i32; 2],
    pub tex_size: [u32; 2],
}

pub struct InkDiffuse {
    app_view: AppView,
    flow_type: FlowType,
    lattice: wgpu::Extent3d,
    collide_stream_node: CollideStreamNode,
    interact_node: ComputeNode,
    touch_uniform: TouchUniform,
    touch_uniform_buffer: BufferObj,
    is_interacting: bool,
    particle_node: Box<dyn RenderNode>,
}

impl InkDiffuse {
    pub fn new(app_view: AppView) -> Self {
        let mut app_view = app_view;
        let flow_type = FlowType::Ink;

        let lattice = Extent3d { width: 200, height: 150, depth: 1 };
        let particle_num = Extent3d { width: 0, height: 0, depth: 0 };
        let threadgroup_count: (u32, u32) = ((lattice.width + 15) / 16, (lattice.height + 15) / 16);

        let mut encoder = app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let lattice_info_data = init_data(lattice.width, lattice.height, flow_type);
        let scalar_lattice_size = (lattice.width * lattice.height * 4) as wgpu::BufferAddress;

        let lattice_buffer = BufferObj::create_empty_storage_buffer(&mut app_view.device, scalar_lattice_size * 9);
        let info_buffer = BufferObj::create_storage_buffer(&mut app_view.device, &mut encoder, &lattice_info_data);

        let diffuse_scalar_buffer = BufferObj::create_empty_storage_buffer(&mut app_view.device, scalar_lattice_size);
        let temp_scalar_buffer = BufferObj::create_empty_storage_buffer(&mut app_view.device, scalar_lattice_size);

        let macro_buffer = BufferObj::create_empty_storage_buffer(&mut app_view.device, scalar_lattice_size * 4);

        let (d2q9_uniform_data, fluid_uniform_data) =
            fluid_uniform(lattice, particle_num, flow_type, &app_view.sc_desc);
        let uniform_buf0 = BufferObj::create_uniform_buffer(&mut app_view.device, &mut encoder, &d2q9_uniform_data);
        let uniform_buf1 = BufferObj::create_uniform_buffer(&mut app_view.device, &mut encoder, &fluid_uniform_data);

        let touch_uniform = TouchUniform { touch_point: [9, 9], lt_lattice: [1, 1], tex_size: [16, 16] };
        let touch_uniform_buffer = BufferObj::create_uniform_buffer(&mut app_view.device, &mut encoder, &touch_uniform);

        let base_buffers: Vec<&BufferObj> =
            vec![&lattice_buffer, &temp_scalar_buffer, &info_buffer, &macro_buffer, &diffuse_scalar_buffer];

        let (texture_buffer, ..) = idroid::texture::from_path(
            FileSystem::new(env!("CARGO_MANIFEST_DIR")).get_texture_file_path("brush0.png"),
            &mut app_view.device,
            &mut encoder,
            true,
            true,
        );

        let interact_node = ComputeNode::new(
            &mut app_view.device,
            (16, 16),
            vec![&uniform_buf0, &uniform_buf1, &touch_uniform_buffer],
            base_buffers.clone(),
            vec![(&texture_buffer, true)],
            ("optimized_mem_lbm/ink/interact", env!("CARGO_MANIFEST_DIR")),
        );

        let collide_stream_node = CollideStreamNode::new(
            &mut app_view.device,
            &mut encoder,
            lattice,
            vec![&uniform_buf0, &uniform_buf1, &touch_uniform_buffer],
            base_buffers.clone(),
            "optimized_mem_lbm/ink/collide",
            "optimized_mem_lbm/ink/stream",
        );

        let particle_node: Box<dyn RenderNode> = Box::new(PigmentDiffuseRenderNode::new(
            &app_view.sc_desc,
            &mut app_view.device,
            &mut encoder,
            &macro_buffer,
            &diffuse_scalar_buffer,
            flow_type,
            lattice,
            particle_num,
        ));

        let mut init_node = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            vec![&uniform_buf0, &uniform_buf1, &touch_uniform_buffer],
            base_buffers.clone(),
            vec![],
            ("optimized_mem_lbm/ink/init", env!("CARGO_MANIFEST_DIR")),
        );
        init_node.compute(&mut app_view.device, &mut encoder);
        app_view.queue.submit(&[encoder.finish()]);

        InkDiffuse {
            app_view,
            flow_type,
            lattice,
            collide_stream_node,
            interact_node,
            touch_uniform,
            touch_uniform_buffer,
            is_interacting: true,
            particle_node,
        }
    }
}

impl SurfaceView for InkDiffuse {
    fn scale(&mut self, _scale: f32) {}

    fn touch_moved(&mut self, position: idroid::math::Position) {
        let (scale_x, scale_y) = self.app_view.normalize_touch_point(position.x, position.y);
        let (lattice_x, lattice_y) =
            ((scale_x * self.lattice.width as f32) as i32, (scale_y * self.lattice.height as f32) as i32);
        self.touch_uniform = TouchUniform {
            touch_point: [lattice_x, lattice_y],
            lt_lattice: [lattice_x - 8, lattice_y - 8],
            tex_size: [16, 16],
        };
        self.is_interacting = true;
    }

    fn resize(&mut self) {
        self.app_view.update_swap_chain();
    }

    fn enter_frame(&mut self) {
        let mut encoder = self.app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        if self.is_interacting {
            self.touch_uniform_buffer.update_buffer(&mut encoder, &mut self.app_view.device, &self.touch_uniform);
        }
        {
            let mut cpass = encoder.begin_compute_pass();
            if self.is_interacting {
                self.interact_node.dispatch(&mut cpass);
                self.is_interacting = false;
            }
            self.collide_stream_node.dispatch(&mut cpass);
            self.particle_node.dispatch(&mut cpass);
        }

        let frame = self.app_view.swap_chain.get_next_texture().expect("swap_chain.get_next_texture() timeout");
        self.particle_node.begin_render_pass(&mut self.app_view.device, &frame, &mut encoder);

        self.app_view.queue.submit(&[encoder.finish()]);
    }
}

pub fn init_data(nx: u32, ny: u32, flow_type: FlowType) -> Vec<LatticeInfo> {
    let mut info: Vec<LatticeInfo> = vec![];

    for j in 0..ny {
        for i in 0..nx {
            info.push(LatticeInfo {
                material: setup_lattice(i, j, nx, ny, flow_type) as f32,
                diffuse_step_count: 0.0,
                iter: 0.0,
                threshold: 0.0,
            });
        }
    }

    info
}
