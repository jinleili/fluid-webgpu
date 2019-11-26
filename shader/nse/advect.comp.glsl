layout(local_size_x = 16, local_size_y = 16) in;

#include "nse/layout_and_fn.glsl"

vec2 src_2f(int u, int v) {
  uint uu = clamp(0, u, lattice_num.x - 1);
  uint uv = clamp(0, v, lattice_num.x - 1);
  return pre_velocity[uv * lattice_num.x + uu];
}

#include "func/bilinear_interpolate_f2.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }

  uint cur_index = indexOfLattice(uv);
  vec2 past_uv = gl_GlobalInvocationID.xy -
                        (pre_velocity[cur_index] * 0.016 / lattice_size);
  velocity[cur_index] = bilinear_interpolate_2f(past_uv);
}