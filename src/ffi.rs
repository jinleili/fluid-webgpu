
use crate::{lbm, optimized_mem_lbm, FlowType};

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_poiseuille_view(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::poiseuille);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_lip_driven_cavity(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::lid_driven_cavity);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn create_pigments_diffuse(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::pigments_diffuse);
    idroid::box_obj(obj)
}
