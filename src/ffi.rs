#[cfg(target_os = "ios")]
use crate::{lbm, optimized_mem_lbm, FlowType};

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn poiseuille_view(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::Poiseuille);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn lip_driven_cavity(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::LidDrivenCavity);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn optimized_poiseuille_view(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = optimized_mem_lbm::D2Q9Flow::new(rust_view, FlowType::Poiseuille);
    idroid::box_obj(obj)
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn pigments_diffuse(view: uni_view::AppViewObj) -> *mut libc::c_void {
    let rust_view = uni_view::AppView::new(view);
    let obj = lbm::D2Q9Flow::new(rust_view, FlowType::PigmentsDiffuse);
    idroid::box_obj(obj)
}
