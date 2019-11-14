use idroid::math::Size;
use idroid::texture;
use idroid::SurfaceView;

use uni_view::{AppView, GPUContext};

use super::RenderNode;
use crate::FieldUniform;
use idroid::node::ComputeNode;

use rand::Rng;

pub struct VectorFieldView {
    app_view: AppView,
    field_node: ComputeNode,
    particle_count: u32,
    particle_node: ComputeNode,
    render_node: RenderNode,
}

fn init_particle_data(num: Size, screen_size: (f32, f32)) -> Vec<f32> {
    let mut data: Vec<f32> = vec![];
    let mut rng = rand::thread_rng();
    let step_x = screen_size.0 / (num.x - 1) as f32;
    let step_y = screen_size.1 / (num.y - 1) as f32;
    for x in 0..num.x {
        let pixel_x = step_x * x as f32;
        for y in 0..num.y {
            data.push(pixel_x + rng.gen_range(-3.0, 3.0));
            data.push(step_y * y as f32 + rng.gen_range(-3.0, 3.0));
            data.push(0.0);
        }
    }

    data
}

impl VectorFieldView {
    pub fn new(app_view: AppView) -> Self {
        let mut app_view = app_view;
        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let screen_size = (app_view.sc_desc.width as f32, app_view.sc_desc.height as f32);
        let particle_num = Size::new(100, 100);
        let field_grid = (5_u32, 4_u32);

        let pu = FieldUniform {
            canvas_size: [screen_size.0, screen_size.1],
            particle_size: [particle_num.x as i32, particle_num.y as i32],
            field_size: [field_grid.0 as i32, field_grid.1 as i32, 2, 0],
            use_canvas_0: [0.0, 0.0],
        };

        let uniform_size = std::mem::size_of::<FieldUniform>() as wgpu::BufferAddress;
        let uniform_buf = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            pu.clone(),
            uniform_size,
        );

        let init_data = init_particle_data(particle_num, screen_size);

        let field_buffer_range = (field_grid.0 * field_grid.1 * 2 * 4) as wgpu::BufferAddress;
        let field_buffer = app_view.device.create_buffer(&wgpu::BufferDescriptor {
            size: field_buffer_range,
            usage: wgpu::BufferUsage::STORAGE,
        });

        let particle_buffer_range = (particle_num.count() * 3 * 4) as wgpu::BufferAddress;
        let (particle_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &init_data,
            particle_buffer_range,
        );

        let particle_node = ComputeNode::new(
            &mut app_view.device,
            particle_num.into(),
            &uniform_buf,
            uniform_size,
            vec![&particle_buffer, &field_buffer],
            vec![particle_buffer_range, field_buffer_range],
            vec![],
            ("particle/particle", env!("CARGO_MANIFEST_DIR")),
        );

        let render_node = RenderNode::new(
            &app_view.sc_desc,
            &mut app_view.device,
            &mut app_view.queue,
            &uniform_buf,
            uniform_size,
            vec![&particle_buffer],
            vec![particle_buffer_range],
            particle_num,
        );

        let mut field_node = ComputeNode::new(
            &mut app_view.device,
            field_grid,
            &uniform_buf,
            uniform_size,
            vec![&field_buffer],
            vec![field_buffer_range],
            vec![],
            ("particle/vector_field", env!("CARGO_MANIFEST_DIR")),
        );
        field_node.compute(&mut app_view.device, &mut encoder);
        app_view.queue.submit(&[encoder.finish()]);

        VectorFieldView {
            app_view,
            particle_count: particle_num.count(),
            particle_node,
            field_node,
            render_node,
        }
    }
}

impl SurfaceView for VectorFieldView {
    fn scale(&mut self, scale: f32) {}

    fn touch_moved(&mut self, _position: idroid::math::Position) {}

    fn resize(&mut self) {
        self.app_view.update_swap_chain();
    }

    fn enter_frame(&mut self) {
        let mut encoder = self
            .app_view
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.particle_node.compute(&mut self.app_view.device, &mut encoder);
        {
            self.render_node.fade_node.compute(&mut self.app_view.device, &mut encoder);
        }
        let frame = self
            .app_view
            .swap_chain
            .get_next_texture()
            .expect("swap_chain.get_next_texture() timeout");
        {
            self.render_node.begin_render_pass(&frame.view, &mut encoder);
        }

        self.app_view.queue.submit(&[encoder.finish()]);
    }
}
