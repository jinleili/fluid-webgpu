layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  vec2 velocity = vec2(0.0);
  float rho = 1.0;
  uint field_index = fieldIndex(uv);

  int material = lattice_info[field_index].material;
  if (isBounceBackCell(material)) {
    rho = 0.0;
    for (uint i = 0; i < 9; i++) {
      collid_streaming_cells[latticeIndex(uv) + i] = 0.0;
    }
  }
  macro_info[field_index].velocity = velocity;
  macro_info[field_index].rho = rho;
  // macro_info[field_index].velocity = vec2(1.0, 2.0);
  // macro_info[field_index].rho = 3.0;
  temp_scalar_cells[field_index] = 0.0;

  // use equilibrium distribution as init value
  if (isBulkFluidCell(material)) {
    //   float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
    for (uint i = 0; i < 9; i++) {
      // float feq = equilibrium(velocity, rho, i, usqr);
      collid_streaming_cells[latticeIndex(uv) + i] = w(i);
    }
  }
}