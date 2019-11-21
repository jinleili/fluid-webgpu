layout(local_size_x = 16, local_size_y = 16) in;

#include "nse/layout_and_fn.glsl"

float d(ivec2 uv) { return divergence[uv.x + uv.y * int(lattice_num.x)]; }

float p(int x, int y) { return pressure[x + y * int(lattice_num.x)]; }

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num.x) || uv.y >= int(lattice_num.y)) {
    return;
  }
  // Perform a single iteration of the Jacobi method in order to solve for pressure.
  pressure[uv.x + uv.y * int(lattice_num.x)] =
      0.25 * (d(uv) + p(uv.x + 2, uv.y) + p(uv.x - 2, uv.y) +
              p(uv.x, uv.y + 2) + p(uv.x, uv.y - 2));
}