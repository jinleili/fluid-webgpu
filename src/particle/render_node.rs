use idroid::geometry::plane::Plane;
use idroid::math::{Size, ViewSize};

use idroid::node::{BindingGroupSettingNode, ComputeNode, ImageViewNode};

pub struct RenderNode {
    view_size: ViewSize,
    particle_vertex_buf: wgpu::Buffer,
    particle_vertex_count: usize,
    setting_node: BindingGroupSettingNode,
    pipeline: wgpu::RenderPipeline,
    particle_canvas: wgpu::TextureView,
    pub fade_node: ComputeNode,
    image_view_node: ImageViewNode,
}

impl RenderNode {
    pub fn new(
        sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device, _queue: &mut wgpu::Queue,
        uniform_buffer: &wgpu::Buffer, uniform_buffer_range: wgpu::BufferAddress,
        buffers: Vec<&wgpu::Buffer>, buffers_range: Vec<wgpu::BufferAddress>, particle_num: Size,
    ) -> Self {
        let view_size = ViewSize { width: sc_desc.width as f32, height: sc_desc.height as f32 };

        let particle_canvas = idroid::texture::empty(
            device,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::Extent3d { width: sc_desc.width, height: sc_desc.height, depth: 1 },
        );

        let vertex_data = self::particle_vertex_data(particle_num);
        let particle_vertex_count = vertex_data.len();
        let particle_vertex_size = std::mem::size_of::<i32>();
        let particle_vertex_buf = device
            .create_buffer_mapped(particle_vertex_count, wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertex_data);

        let setting_node = BindingGroupSettingNode::new(
            device,
            vec![uniform_buffer],
            vec![uniform_buffer_range],
            buffers,
            buffers_range,
            vec![],
            vec![],
            vec![wgpu::ShaderStage::VERTEX, wgpu::ShaderStage::VERTEX, wgpu::ShaderStage::VERTEX],
        );

        // Create the render pipeline
        let shader = idroid::shader::Shader::new(
            "particle/vector_field_rendering",
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
            primitive_topology: wgpu::PrimitiveTopology::PointList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: idroid::utils::color_blend(),
                alpha_blend: idroid::utils::alpha_blend(),
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: particle_vertex_size as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &vec![wgpu::VertexAttributeDescriptor {
                    shader_location: 0,
                    format: wgpu::VertexFormat::Int,
                    offset: 0,
                }],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mvp = idroid::utils::MVPUniform {
            mvp_matrix: idroid::utils::matrix_helper::fullscreen_mvp(sc_desc),
        };
        let image_view_node = ImageViewNode::new(
            sc_desc,
            device,
            (&particle_canvas, false),
            mvp,
            ("none", env!("CARGO_MANIFEST_DIR")),
        );

        let threadgroup_count = ((sc_desc.width + 15) / 16, (sc_desc.height + 15) / 16);
        let fade_node = ComputeNode::new(
            device,
            threadgroup_count,
            uniform_buffer,
            uniform_buffer_range,
            vec![],
            vec![],
            vec![&particle_canvas],
            ("particle/fade_out", env!("CARGO_MANIFEST_DIR")),
        );

        RenderNode {
            view_size,
            particle_vertex_buf,
            particle_vertex_count,
            setting_node,
            pipeline,
            particle_canvas,
            fade_node,
            image_view_node,
        }
    }

    pub fn begin_render_pass(&self, frame: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.particle_canvas,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: idroid::utils::clear_color(),
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.setting_node.bind_group, &[]);
            rpass.set_vertex_buffers(0, &[(&self.particle_vertex_buf, 0)]);
            rpass.draw(0..self.particle_vertex_count as u32, 0..1);
        }
        {
            self.image_view_node.begin_render_pass(frame, encoder);
        }
    }
}

pub fn particle_vertex_data(num: Size) -> Vec<i32> {
    let mut list = vec![];

    for x in 0..num.x {
        for y in 0..num.y {
            list.push((y * num.x + x) as i32);
        }
    }
    list
}
