use super::NSEFluidUniform;
use idroid::node::ComputeNode;
use idroid::SurfaceView;
use wgpu::Extent3d;
use uni_view::{AppView, GPUContext};

pub struct Smoke2D {
    app_view: AppView,
    advert_node0: ComputeNode,
    advert_node1: ComputeNode,
    swap: i32,
}

impl Smoke2D {
    pub fn new(app_view: AppView) -> Self {
        let mut app_view = app_view;
        let swap = 0_i32;

        let lattice_num = (100, 75);
        let threadgroup_count: (u32, u32) = ((lattice_num.0 + 15) / 16, (lattice_num.1 + 15) / 16);
        let lattice = Extent3d { width: lattice_num.0, height: lattice_num.1, depth: 1 };

        let mut encoder =
            app_view.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        // velocity buffer 所占字节数
        let velocity_buf_range = (lattice.width * lattice.height * 2 * 4) as wgpu::BufferAddress;
        // scalar field buffer 的字节数
        let scalar_buf_range = (lattice.width * lattice.height * 1 * 4) as wgpu::BufferAddress;
        let (vector_data, scalar_data) = init_data(lattice.width, lattice.height);

        let (velocity0_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &vector_data,
            velocity_buf_range,
        );
        let (velocity1_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &vector_data,
            velocity_buf_range,
        );

        let (divergence_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &scalar_data,
            scalar_buf_range,
        );
        let (pressure_buffer, _) = idroid::utils::create_storage_buffer(
            &mut app_view.device,
            &mut encoder,
            &scalar_data,
            scalar_buf_range,
        );

        let uniform_size = std::mem::size_of::<NSEFluidUniform>() as wgpu::BufferAddress;
        let uniform_buf = idroid::utils::create_uniform_buffer2(
            &mut app_view.device,
            &mut encoder,
            get_uniform(lattice, &app_view.sc_desc),
            uniform_size,
        );

        let advert_node0 = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            &uniform_buf,
            uniform_size,
            vec![&velocity0_buffer, &velocity1_buffer, &divergence_buffer, &pressure_buffer],
            vec![velocity_buf_range, velocity_buf_range, scalar_buf_range, scalar_buf_range],
            vec![],
            ("nse/advect", env!("CARGO_MANIFEST_DIR")),
        );
        let advert_node1 = ComputeNode::new(
            &mut app_view.device,
            threadgroup_count,
            &uniform_buf,
            uniform_size,
            vec![&velocity0_buffer, &velocity1_buffer, &divergence_buffer, &pressure_buffer],
            vec![velocity_buf_range, velocity_buf_range, scalar_buf_range, scalar_buf_range],
            vec![],
            ("nse/advect", env!("CARGO_MANIFEST_DIR")),
        );
        Smoke2D { app_view, swap, advert_node0, advert_node1 }
    }
}

impl SurfaceView for Smoke2D {
    fn scale(&mut self, _scale: f32) {}

    fn touch_moved(&mut self, _position: idroid::math::Position) {}

    fn resize(&mut self) {
        self.app_view.update_swap_chain();
    }

    fn enter_frame(&mut self) {
        self.swap += 1;
        if self.swap % 10 != 0 {
            return;
        }

        let mut encoder = self
            .app_view
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        self.advert_node0.compute(&mut self.app_view.device, &mut encoder);

        self.app_view.queue.submit(&[encoder.finish()]);
    }
}

fn init_data(nx: u32, ny: u32) -> (Vec<[f32; 2]>, Vec<f32>) {
    let mut scalar_data: Vec<f32> = vec![];
    let mut vector_data: Vec<[f32; 2]> = vec![];
    for j in 0..ny {
        for i in 0..nx {
            vector_data.push([0.0, 0.0]);
            scalar_data.push(0.0);
        }
    }
    (vector_data, scalar_data)
}

fn get_uniform(lattice: Extent3d, sc_desc: &wgpu::SwapChainDescriptor) -> NSEFluidUniform {
    let uniform = NSEFluidUniform {
        lattice_size: [2.0 / lattice.width as f32, 2.0 / lattice.height as f32],
        pixel_distance: [2.0 / sc_desc.width as f32, 2.0 / sc_desc.height as f32],
        lattice_num: [lattice.width, lattice.height],
        canvas_size: [sc_desc.width, sc_desc.height],
    };
    uniform
}
