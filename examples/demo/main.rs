extern crate idroid;
use idroid::{math::Position, SurfaceView};

extern crate uni_view;
use uni_view::AppView;

extern crate fluid_webgpu;
use fluid_webgpu::{PoiseuilleFlow, VectorFieldView};

extern crate lazy_static;
extern crate objc;

fn main() {
    use winit::event::{
        ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    };
    use winit::{event_loop::EventLoop, window::Window};

    env_logger::init();

    let events_loop = EventLoop::new();
    let window = Window::new(&events_loop).unwrap();
    // window.set_max_dimensions(Some((400_u32, 700_u32).into()));
    // window.set_inner_size((600_u32, 600_u32).into());
    window.set_max_inner_size(Some((800_u32, 1850_u32).into()));
    window.set_title("brush");

    let v = AppView::new(window);

    let mut surface_view = PoiseuilleFlow::new(v);
    // let mut surface_view = VectorFieldView::new(v);
    // let mut surface_view = fluid_webgpu::ReadWriteTest::new(v);
    // winit 0.20.0-alpha3 不会主动触发 WindowEvent::Resized 事件了

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

    // let triangle = idroid::Triangle::new()
}

// 找到第一个大于 0 的高度所在的索引位置： i
// 遍历找到 j,  arr[j] >= arr[i], 或者 arr[j] < arr[i] 且大于 arr[i -> j] 区间的值
// i 与 j, 取它俩之间的最小高度对应的索引： k = arr[i] > arr[j] ? j : i;
// step =（j - i) * arr[k] - sum(arr[i + 1] -> arr[j - 1]) - arr[k];
// 递归（j 就是下一次遍历时的 i)累加 step 值

fn water() {
    let arr: [i32; 12] = [0, 1, 0, 2, 1, 0, 1, 3, 2, 1, 2, 1];
    let i: usize = 1;
    let sum = step(i, &arr);
    println!("注水总量是： {}", sum);
}

fn step(i: usize, arr: &[i32; 12]) -> i32 {
    if i >= arr.len() {
        return 0;
    }
    let mut j = i;
    for index in i..arr.len() {
        if arr[index] >= arr[i] {
            j = index;
            break;
        }
    }
    // 没有找到比 i 大的，则找小的里面最大的
    if j == i {
        j = i + 1;
        for index in j..arr.len() {
            if arr[index] >= arr[j] {
                j = index;
            }
        }
    }

    let k = if arr[i] > arr[j] { j } else { i };
    let res: i32 = (j - i) as i32 * arr[k] - sum(i + 1, j - 1, arr) - arr[k];

    return res + step(j, arr);
}

fn sum(i: usize, j: usize, arr: &[i32]) -> i32 {
    let mut s = 0;
    if i < j {
        for index in i..=j {
            s += arr[index];
        }
    }
    return s;
}
