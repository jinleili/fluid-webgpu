layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/ink_layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);  
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  uint field_index = fieldIndex(uv);
  diffuse[field_index] = 0.0;
  temp_scalar_cells[field_index] = 0.0;
  // for (uint i = 0; i < 9; i++) {
  //   collid_streaming_cells[latticeIndex(uv) + i] = w(i);
  // }
}