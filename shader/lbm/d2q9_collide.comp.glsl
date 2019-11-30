layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/code_block/fluid_layout_and_fn.glsl"

void update_collide(ivec2 uv, int direction, float collide) {
  collidingCells[latticeIndex(uv) + direction] = collide;
}

// update macroscope velocity, dencity...
void update_macro(ivec2 uv, vec2 velocity, float rho) {
  int destIndex = uv.x + uv.y * int(lattice_num.x);
  macro_info[destIndex].xyz = vec3(velocity, rho);
}

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  int material = int(macro_info[fieldIndex(uv)].w);
  // at boundary lattice, not need calculate collide and stream
  if (isBounceBackCell(material) || isLidDrivenCell(material)) {
    return;
  }

  float f_i[9];
  vec2 velocity = vec2(0.0);
  float rho = 0.0;

  for (int i = 0; i < 9; i++) {
    f_i[i] = streamingCells[latticeIndex(uv) + i];
    rho += f_i[i];
    velocity += e(i) * f_i[i];
  }
  velocity = velocity / rho;

  if (isOutflowCell(material)) {
    // outflow restore dencity
    rho = 1.0;
  } else if (isInflowCell(material)) {
    // inflow add extra force
    velocity = vec2(0.1, 0.00);
  }
  update_macro(uv, velocity, rho);

  // Collision step: fout = fin - omega * (fin - feq)
  float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
  for (int i = 0; i < 9; i++) {
    float collide =
        f_i[i] - omega() * (f_i[i] - equilibrium(velocity, rho, i, usqr));

    update_collide(uv, i, collide);
  }
}
