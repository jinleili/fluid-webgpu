use super::{ParticleUniform, PixelInfo};
use idroid::geometry::plane::Plane;
use idroid::math::ViewSize;
use idroid::node::BindingGroupSettingNode;
use idroid::node::ComputeNode;
use idroid::utils::MVPUniform;
use idroid::vertex::{Pos, PosTex};

use rand::Rng;
use std::vec::Vec;

use super::Particle;

pub struct RenderNode {
    view_size: ViewSize,
    particle_vertex_buf: wgpu::Buffer,
    particle_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,

    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,

    pub particle_node: ComputeNode,
    pub fade_node: ComputeNode,

    depth_texture_view: wgpu::TextureView,
}

impl RenderNode {
    pub fn new(
        sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder, uniform_buffers: Vec<&wgpu::Buffer>,
        uniform_buffer_ranges: Vec<wgpu::BufferAddress>, field_buffer: &wgpu::Buffer,
        field_buffer_range: wgpu::BufferAddress, particle: wgpu::Extent3d,
    ) -> Self {
        let view_size = ViewSize { width: sc_desc.width as f32, height: sc_desc.height as f32 };

        let canvas_data = init_canvas_data(sc_desc);
        let canvas_buffer_size = (sc_desc.width * sc_desc.height * std::mem::size_of::<PixelInfo>() as u32) as wgpu::BufferAddress;
        let (canvas_buffer, _) =
            idroid::utils::create_storage_buffer(device, encoder, &canvas_data, canvas_buffer_size);
        let vertex_data = self::particle_vertex_data(particle);
        let particle_count = vertex_data.len();
        let particle_vertex_size = std::mem::size_of::<i32>();
        let particle_vertex_buf = device
            .create_buffer_mapped(particle_count, wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertex_data);

        let init_data = init_particle_data(particle);
        let particle_buffer_range =
            (particle.width * particle.height * std::mem::size_of::<Particle>() as u32)
                as wgpu::BufferAddress;
        let (particle_buffer, _) = idroid::utils::create_storage_buffer(
            device,
            encoder,
            &init_data,
            particle_buffer_range,
        );

        let threadgroup_count = ((particle.width + 15) / 16, (particle.height + 15) / 16);

        let particle_node = ComputeNode::new(
            device,
            threadgroup_count,
            uniform_buffers[1],
            uniform_buffer_ranges[1],
            vec![&particle_buffer, field_buffer, &canvas_buffer],
            vec![particle_buffer_range, field_buffer_range, canvas_buffer_size],
            vec![],
            ("lbm/particle_move", env!("CARGO_MANIFEST_DIR")),
        );

        let uniform_size = std::mem::size_of::<MVPUniform>() as wgpu::BufferAddress;
        let uniform_buf = idroid::utils::create_uniform_buffer2(
            device,
            encoder,
            MVPUniform { mvp_matrix: idroid::utils::matrix_helper::fullscreen_mvp(sc_desc) },
            uniform_size,
        );

        let setting_node = BindingGroupSettingNode::new(
            device,
            vec![&uniform_buf, uniform_buffers[1]],
            vec![uniform_size, uniform_buffer_ranges[1]],
            vec![&canvas_buffer],
            vec![canvas_buffer_size],
            vec![],
            vec![],
            vec![
                wgpu::ShaderStage::VERTEX,
                wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                wgpu::ShaderStage::FRAGMENT,
            ],
        );

        // Create the vertex and index buffers
        let (vertex_data, index_data) = Plane::new(1, 1).generate_vertices();
        let vertex_buf = device
            .create_buffer_mapped(vertex_data.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertex_data);

        let index_buf = device
            .create_buffer_mapped(index_data.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&index_data);

        // Create the render pipeline
        let shader = idroid::shader::Shader::new(
            "lbm/particle_presenting",
            device,
            env!("CARGO_MANIFEST_DIR"),
        );
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &setting_node.pipeline_layout,
            vertex_stage: shader.vertex_stage(),
            fragment_stage: shader.fragment_stage(),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: idroid::utils::color_blend(),
                alpha_blend: idroid::utils::alpha_blend(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(idroid::depth_stencil::create_state_descriptor()),
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: std::mem::size_of::<PosTex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &PosTex::attri_descriptor(0),
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        println!("uniform_buffer_ranges[1]: {}", uniform_buffer_ranges[1]);

        let fade_node = ComputeNode::new(
            device,
            ((sc_desc.width + 15) / 16, (sc_desc.height + 15) / 16),
            uniform_buffers[1],
            uniform_buffer_ranges[1],
            vec![&canvas_buffer],
            vec![canvas_buffer_size],
            vec![],
            ("lbm/fade_out", env!("CARGO_MANIFEST_DIR")),
        );
        let depth_texture_view = idroid::depth_stencil::create_depth_texture_view(sc_desc, device);

        RenderNode {
            view_size,
            particle_vertex_buf,
            particle_count,
            setting_node,
            pipeline,
            depth_texture_view,
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            particle_node,
            fade_node,
        }
    }

    pub fn begin_render_pass(
        &mut self, device: &mut wgpu::Device, frame: &wgpu::SwapChainOutput,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        {
            // 先执行淡出
            self.fade_node.compute(device, encoder);
        }
        {
            // move particle
            self.particle_node.compute(device, encoder);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color { r: 0.1, g: 0.15, b: 0.17, a: 1.0 },
                }],
                depth_stencil_attachment: Some(
                    idroid::depth_stencil::create_attachment_descriptor(&self.depth_texture_view),
                ),
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
            rpass.set_index_buffer(&self.index_buf, 0);
            rpass.set_vertex_buffers(0, &[(&self.vertex_buf, 0)]);
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }
    }
}

fn particle_vertex_data(num: wgpu::Extent3d) -> Vec<i32> {
    let mut list = vec![];

    for x in 0..num.width {
        for y in 0..num.height {
            list.push((y * num.width + x) as i32);
        }
    }
    list
}

fn init_particle_data(num: wgpu::Extent3d) -> Vec<Particle> {
    let mut data: Vec<Particle> = vec![];
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
            data.push(Particle {
                pos: pos,
                pos_initial: pos,
                life_time: rng.gen_range(0, 60) as f32,
                fade: 1.0,
            });
        }
    }

    data
}

fn init_canvas_data(sc_desc: &wgpu::SwapChainDescriptor) -> Vec<PixelInfo> {
    let mut data: Vec<PixelInfo> = vec![];
    for _ in 0..sc_desc.width {
        for _ in 0..sc_desc.height {
            data.push(PixelInfo{ alpha: 0.0, speed: 0.0, rho: 0.0});
        }
    }
    data
}
