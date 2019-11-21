mod smoke_2d;
pub use smoke_2d::Smoke2D;

#[repr(C)]
#[derive(Copy, Clone)]
struct NSEFluidUniform {
    // lattice 在正规化坐标空间的大小
    lattice_size: [f32; 2],
    // 正规化坐标空间里，一个像素对应的距离值
    pixel_distance: [f32; 2],

    lattice_num: [u32; 2],
    // 画布像素尺寸
    canvas_size: [u32; 2],
}
