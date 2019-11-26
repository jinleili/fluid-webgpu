extern crate libc;
pub use idroid::utils::{depth_stencil, matrix_helper};
pub use uni_view::*;

pub mod lbm;

mod nse;
pub use nse::Smoke2D;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4Uniform {
    info: [f32; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PicInfoUniform {
    info: [i32; 4],
    // only for requested 256 alignment: (256 - 16) / 4 = 60
    any: [i32; 60],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PicInfoUniform2 {
    info: [i32; 4],
    threshold: [f32; 4],
    any: [i32; 56],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldUniform {
    canvas_size: [f32; 2],
    particle_size: [i32; 2],
    field_size: [i32; 4],
    use_canvas_0: [f32; 2],
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_poiseuille_view(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, lbm::FlowType::poiseuille);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_lip_driven_cavity(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, lbm::FlowType::lid_driven_cavity);
    idroid::box_obj(obj)
}
