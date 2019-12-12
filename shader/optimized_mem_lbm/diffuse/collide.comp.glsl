layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

layout(set = 0, binding = 6) buffer DiffuseBuffer0 { float diffuse_cells[]; };
layout(set = 0, binding = 7) buffer DiffuseBuffer2 { float diffuse[]; };

layout(set = 1, binding = 0) uniform Q9DirectionUniform {
  uint direction;
  vec4 any[15];
};

// diffuse relaxation time
const float DIFFUSE_OMEGA = 1.0 / (3.0 * 0.6 + 0.5);

float diffuse_feq(vec2 velocity, float rho, uint direction) {
  return rho * w(direction) * (1.0 + 3.0 * dot(e(direction), velocity));
}

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  uint field_index = fieldIndex(uv);
  int material = int(lattice_info[field_index].material);
  // at boundary lattice, not need calculate collide and stream
  if (isBounceBackCell(material) || isLidDrivenCell(material)) {
    return;
  }

  vec2 velocity =
      vec2(macro_info[field_index * 3], macro_info[field_index * 3 + 1]);
  // Collision step: fout = fin - omega * (fin - feq)
  float f_i = diffuse_cells[latticeIndex(uv) + direction];
  if (direction == 0) {
    // update macroscope concentration ...
    float rho = 0.0;
    for (uint i = 0; i < 9; i++) {
      rho += diffuse_cells[latticeIndex(uv) + i];
    }

    diffuse[field_index] = rho;
    // rest population on lattice center not need stream
    diffuse_cells[latticeIndex(uv)] =
        f_i -
        DIFFUSE_OMEGA * (f_i - diffuse_feq(velocity * 7.0, rho, direction));
  } else {
    float rho = diffuse[field_index];
    temp_scalar_cells[field_index] =
        f_i -
        DIFFUSE_OMEGA * (f_i - diffuse_feq(velocity * 7.0, rho, direction));
  }
}
