layout(local_size_x = 1, local_size_y = 1) in;

#include "optimized_mem_lbm/code_block/ink_layout_and_fn.glsl"
layout(set = 0, binding = 8, r8) uniform image2D brush;

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  ivec2 lattice_uv = uv + lt_lattice;
  if (lattice_uv.x < 0 || lattice_uv.x >= int(lattice_num.x) || lattice_uv.y < 0 ||
      lattice_uv.y >= int(lattice_num.y)) {
    return;
  }
  float color = imageLoad(brush, uv).r;
  if (color > 0.2) {
    color *= 5.0;

    for (uint i = 0; i < 9; i++) {
      collid_streaming_cells[latticeIndex(uvec2(lattice_uv)) + i] += color * w(i);
    }
  }
}