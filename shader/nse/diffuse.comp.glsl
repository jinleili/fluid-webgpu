layout(local_size_x = 16, local_size_y = 16) in;

#include "nse/layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  float a = 0.016 * 0.02 * float(lattice_num.x * lattice_num.y);
  uint cur_index = indexOfLattice(uv);
  velocity[cur_index] = (pre_velocity[cur_index] +
                         a * (velocity[cur_index - 1] + velocity[cur_index + 1] + 
                         velocity[indexOfLattice(uv.x, uv.y - 1)]) + velocity[indexOfLattice(uv.x, uv.y + 1)])) /
                        (1.0 + 4.0 * a);
}