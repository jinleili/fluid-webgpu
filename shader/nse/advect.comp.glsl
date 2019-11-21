layout(local_size_x = 16, local_size_y = 16) in;

#include "nse/layout_and_fn.glsl"

vec2 srcData(int u, int v) { return pre_velocity[v * lattice_num.x + u]; }

vec2 bilinear_interpolate_2f(vec2 uv, uvec2 max_uv) {
  int minX = int(floor(uv.x));
  int minY = int(floor(uv.y));
  int valid_min_x = max(0, minX);
  int valid_min_y = max(0, minY);
  int min_plus1_x = min(minX + 1, int(max_uv.x));
  int min_plus1_y = min(minY + 1, int(max_uv.y));

  float fx = uv.x - float(minX);
  float fy = uv.y - float(minY);
  // 插值公式： f(i+u,j+v) = (1-u)(1-v)f(i,j) + (1-u)vf(i,j+1) +
  // u(1-v)f(i+1,j) + uvf(i+1,j+1)
  return srcData(valid_min_x, valid_min_y) * ((1.0 - fx) * (1.0 - fy)) +
         srcData(valid_min_x, min_plus1_y) * ((1.0 - fx) * fy) +
         srcData(min_plus1_x, valid_min_y) * (fx * (1.0 - fy)) +
         srcData(min_plus1_x, min_plus1_y) * (fx * fy);
}

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }

  uint cur_index = indexOfLattice(uv);
  vec2 past_uv = gl_GlobalInvocationID.xy -
                        (pre_velocity[cur_index] * 0.016 / lattice_size);
  velocity[cur_index] = bilinear_interpolate_2f(past_uv, lattice_num);
}