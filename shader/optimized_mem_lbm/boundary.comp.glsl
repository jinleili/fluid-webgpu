layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  int material = lattice_info[fieldIndex(uv)].material;

  // on-grid bounce back
#include "optimized_mem_lbm/code_block/cal_on_grid_bb.glsl"
}