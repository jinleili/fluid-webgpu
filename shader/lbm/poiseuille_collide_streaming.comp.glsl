layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

void update_collide(ivec2 uv, int direction, float collide) {
  collidingCells[indexOfLattice(uv) + direction] = collide;
}

// frome current lattice streaming to neighbour
void streaming_out(ivec2 uv, int direction, float collide) {
  // https://pdfs.semanticscholar.org/e626/ca323a9a8a4ad82fb16ccbbbd93ba5aa98e0.pdf
  // along current direction streaming to neighbour lattice same direction
  ivec2 new_uv = uv + ivec2(e(direction));
  // if not detect coordinate's legality, on iOS will cause error:
  // Execution of the command buffer was aborted due to an error during
  // execution. Ignored (for causing prior/excessive GPU errors) (IOAF code 4)
  if (new_uv.x > 0 && new_uv.x < int(lattice_num.x)) {
    streamingCells[indexOfLattice(new_uv) + direction] = collide;
  }
}

// frome neighbour lattice streaming in current
void streaming_in(ivec2 uv, int direction) {
  ivec2 new_uv = uv - ivec2(e(direction));
  streamingCells[indexOfLattice(uv) + direction] =
      collidingCells[indexOfLattice(new_uv) + direction];
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
  int material = int(macro_info[indexOfFluid(uv)].w);
  // at boundary lattice, not need calculate collide and stream
  if (isBounceBackCell(material)) {
    return;
  }

  float f_i[9];
  vec2 velocity = vec2(0.0);
  float rho = 0.0;

  for (int i = 0; i < 9; i++) {
    f_i[i] = streamingCells[indexOfLattice(uv) + i];
    rho += f_i[i];
    velocity += e(i) * f_i[i];
  }
  velocity = velocity / rho;

  if (isOutflowCell(material)) {
    // outflow restore dencity
    rho = 1.0;
  } else if (isInflowCell(material)) {
    // inflow add extra force
    velocity = vec2(0.1, 0.05);
  }
  update_macro(uv, velocity, rho);

  // Collision step: fout = fin - omega * (fin - feq)
  float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
  for (int i = 0; i < 9; i++) {
    float collide =
        f_i[i] - omega * (f_i[i] - equilibrium(velocity, rho, i, usqr));

    update_collide(uv, i, collide);
    streaming_out(uv, i, collide);
  }
}
