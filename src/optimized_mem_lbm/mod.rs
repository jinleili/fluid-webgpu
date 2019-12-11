use zerocopy::{AsBytes, FromBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct Q9DirectionUniform {
    direction: u32,
    // only for requested 256 alignment: (256 - 4) / 4 = 63
    any0: [i32; 32],
    any1: [i32; 31],
}

mod d2q9_flow;
pub use d2q9_flow::D2Q9Flow;

mod ink_diffuse;
pub use ink_diffuse::InkDiffuse;

mod collide_stream_node;
use collide_stream_node::CollideStreamNode;
