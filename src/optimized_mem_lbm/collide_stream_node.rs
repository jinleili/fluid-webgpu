use super::Q9DirectionUniform;
use idroid::buffer::BufferObj;
use idroid::node::{BindingGroupSettingNode, DynamicBindingGroupNode};
use zerocopy::AsBytes;

pub struct CollideStreamNode {
    common_setting_node: BindingGroupSettingNode,
    step_setting_node: DynamicBindingGroupNode,
    collide_pipeline: wgpu::ComputePipeline,
    stream_pipeline: wgpu::ComputePipeline,
    threadgroup_count: (u32, u32),
}

impl CollideStreamNode {
    pub fn new(
        device: &mut wgpu::Device, lattice: wgpu::Extent3d, uniforms: Vec<&BufferObj>,
        inout_buffer: Vec<&BufferObj>, clide_shader_path: &str, stream_shader_path: &str,
    ) -> Self {
        let mut visibilitys: Vec<wgpu::ShaderStage> = vec![];
        for _ in 0..(uniforms.len() + inout_buffer.len()) {
            visibilitys.push(wgpu::ShaderStage::COMPUTE);
        }
        let common_setting_node = BindingGroupSettingNode::new(
            device,
            uniforms,
            inout_buffer,
            vec![],
            vec![],
            visibilitys,
        );
        let uniform_buffer = BufferObj::create_uniforms_buffer(
            device,
            &[
                Q9DirectionUniform { direction: 0, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 1, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 2, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 3, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 4, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 5, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 6, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 7, any0: [0; 32], any1: [0; 31] },
                Q9DirectionUniform { direction: 8, any0: [0; 32], any1: [0; 31] },
            ]
            .as_bytes(),
        );
        let step_setting_node = DynamicBindingGroupNode::new(
            device,
            vec![&uniform_buffer],
            vec![wgpu::ShaderStage::COMPUTE],
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &common_setting_node.bind_group_layout,
                &step_setting_node.bind_group_layout,
            ],
        });

        let collide_shader = idroid::shader::Shader::new_by_compute(
            clide_shader_path,
            device,
            env!("CARGO_MANIFEST_DIR"),
        );
        let collide_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: collide_shader.cs_stage(),
        });

        let stream_shader = idroid::shader::Shader::new_by_compute(
            stream_shader_path,
            device,
            env!("CARGO_MANIFEST_DIR"),
        );
        let stream_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: stream_shader.cs_stage(),
        });

        let threadgroup_count: (u32, u32) = ((lattice.width + 15) / 16, (lattice.height + 15) / 16);

        CollideStreamNode {
            common_setting_node,
            step_setting_node,
            collide_pipeline,
            stream_pipeline,
            threadgroup_count,
        }
    }

    pub fn dispatch(&mut self, cpass: &mut wgpu::ComputePass) {
        cpass.set_bind_group(0, &self.common_setting_node.bind_group, &[]);
        cpass.set_pipeline(&self.collide_pipeline);
        // set_bind_group doc: https://gpuweb.github.io/gpuweb/
        cpass.set_bind_group(1, &self.step_setting_node.bind_group, &[0]);
        cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);

        for step in 1..=8 {
            cpass.set_bind_group(1, &self.step_setting_node.bind_group, &[256 * step]);

            cpass.set_pipeline(&self.collide_pipeline);
            cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);

            cpass.set_pipeline(&self.stream_pipeline);
            // dispatch 之后 bind_group 一定要重新设置？
            // 观察到的结果是：重新设置得到的执行结果是不一样的
            // cpass.set_bind_group(1, &self.step_setting_node.bind_group, &[256 * step]);
            cpass.dispatch(self.threadgroup_count.0, self.threadgroup_count.1, 1);
        }
    }
}
