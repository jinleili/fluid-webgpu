layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  // Initial velocity with a slight perturbation
  vec2 velocity = vec2(0.0);
  float rho = 1.0;
  int destIndex = indexOfFluid(uv);
  int material = int(macro_info[destIndex].w);
  if (isBounceBackCell(material)) {
    rho = 0.0;
    for (int i = 0; i < 9; i++) {
      collidingCells[indexOfLattice(uv) + i] = 0.0;
      streamingCells[indexOfLattice(uv) + i] = 0.0;
    }
  }
  macro_info[destIndex].xyz = vec3(velocity, rho);

  // use equilibrium distribution as init value
  if (isBulkFluidCell(material)) {
    //   float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
    for (int i = 0; i < 9; i++) {
      // float feq = equilibrium(velocity, rho, i, usqr);
      collidingCells[indexOfLattice(uv) + i] = w(i);
      streamingCells[indexOfLattice(uv) + i] = w(i);
    }
  }
}