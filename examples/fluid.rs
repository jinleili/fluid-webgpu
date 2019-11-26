extern crate idroid;
use idroid::{math::Position, SurfaceView};

extern crate uni_view;
use uni_view::AppView;

extern crate fluid_webgpu;
use fluid_webgpu::{Smoke2D};
use fluid_webgpu::lbm::{FlowType, D2Q9Flow};


static PANIC_MSG: &str = "\n You must pass one of these names: poiseuille, lid_driven_cavity, smoke_2d! \n\n 请输入有效流体名称的其中一个：poiseuille, lid_driven_cavity, smoke_2d! \n";

fn main() {
    use winit::event::{
        ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    };
    use winit::{event_loop::EventLoop, window::Window};

    if std::env::args().len() == 1 || std::env::args().len() > 2 {
        panic!("{}", PANIC_MSG);
    }

    env_logger::init();
    let events_loop = EventLoop::new();
    let window = Window::new(&events_loop).unwrap();
    window.set_inner_size((800_u32, 600_u32).into());
    // window.set_max_inner_size(Some((800_u32, 1850_u32).into()));
    window.set_title("fluid");

    let v = AppView::new(window);

    let mut surface_view: Box<dyn SurfaceView> = {
        let app_name: String = std::env::args().skip(1).next().unwrap();
        if app_name == String::from("smoke_2d") {
            Box::new(Smoke2D::new(v))
        } else if app_name == String::from("poiseuille") {
            Box::new(D2Q9Flow::new(v, FlowType::poiseuille))
        } else if app_name == String::from("lid_driven_cavity"){
            Box::new(D2Q9Flow::new(v, FlowType::lid_driven_cavity))
        } else {
            panic!("{}", PANIC_MSG);
        }

    };

    events_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            winit::event_loop::ControlFlow::Exit
        } else {
            winit::event_loop::ControlFlow::Poll
        };
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(_size), .. } => {
                surface_view.resize();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        println!("{:?}, {}", _x, y);
                    }
                    _ => (),
                },
                WindowEvent::Touch(touch) => {
                    println!("{:?}", touch);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    surface_view.touch_moved(Position::new(position.x as f32, position.y as f32));
                }
                _ => {}
            },
            Event::EventsCleared => {
                surface_view.enter_frame();
            }
            _ => (),
        }
    });
}
