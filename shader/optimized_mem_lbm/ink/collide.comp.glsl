layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/ink_layout_and_fn.glsl"

layout(set = 1, binding = 0) uniform Q9DirectionUniform {
  uint direction;
  // float any[63];
  vec4 any[15];
};

// diffuse relaxation time
const float DIFFUSE_OMEGA = 1.0 / (3.0 * 0.6 + 0.5);

float diffuse_feq(float rho, uint direction) { return rho * w(direction); }

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  uint field_index = fieldIndex(uv);
  LatticeInfo lattice = lattice_info[field_index];
  if (isBounceBackCell(int(lattice.material))) {
    temp_scalar_cells[field_index] = 0.0;
    return;
  }

  float f_i = collid_streaming_cells[latticeIndex(uv) + direction];
  if (direction == 0) {
    temp_scalar_cells[field_index] = 0.0;
    // update macroscope concentration ...
    float rho = 0.0;
    for (uint i = 0; i < 9; i++) {
      rho += collid_streaming_cells[latticeIndex(uv) + i];
    }
    diffuse[field_index] = rho;

    if (isBlockCell(int(lattice.material))) {
      lattice.iter += rho;
      // block lattice change to bulk lattice
      if (lattice.iter > 1.5) {
        lattice.material = 1.0;
        collid_streaming_cells[latticeIndex(uv)] =
            f_i - DIFFUSE_OMEGA * (f_i - diffuse_feq(rho, direction));
      }
      lattice_info[field_index] = lattice;
    } else {
      // if (isBulkFluidCell(lattice.material)) {
      // rest population on lattice center not need stream
      if (rho > 0.7) {
        collid_streaming_cells[latticeIndex(uv)] =
            f_i - DIFFUSE_OMEGA * (f_i - diffuse_feq(rho, direction));
      } else {
        collid_streaming_cells[latticeIndex(uv)] = rho;
      }
    }

  } else {
    float rho = diffuse[field_index];
    if (rho > 0.7) {
      temp_scalar_cells[field_index] =
          f_i - omega() * (f_i - diffuse_feq(rho, direction));
    } else {
      // stop diffuse
      temp_scalar_cells[field_index] = 0.0;
      // collid_streaming_cells[latticeIndex(uv) + direction] = 0.0;
      // collid_streaming_cells[latticeIndex(uv)] = rho;
    }
  }
}
