layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  uint destIndex = fieldIndex(uv);
  int material = int(macro_info[destIndex].w);

  // at boundary lattice, not need calculate collide and stream
  if (isBounceBackCell(material)) {
    return;
  }

  vec2 velocity = vec2(0.0);
  float rho = 0.0;
  for (int i = 0; i < 9; i++) {
    float scalar = collid_streaming_cells[latticeIndex(uv) + i];
    rho += scalar;
    velocity += e(i) * scalar;
  }
  velocity = velocity / rho;

  if (isOutflowCell(material)) {
    // outflow restore dencity
    rho = 1.0;
  } else if (isInflowCell(material)) {
    // inflow add extra force
    velocity = vec2(0.1, 0.00);
  }

  // update macroscope velocity, dencity...
  macro_info[destIndex].xyz = vec3(velocity, rho);
}