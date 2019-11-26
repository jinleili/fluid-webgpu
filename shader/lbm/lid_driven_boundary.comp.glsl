layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

const int bounceBackDirection[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  int material = int(macro_info[indexOfFluid(uv)].w);

  if (isLidDrivenCell(material)) {
    ivec2 streaming_uv = ivec2(uv.x, uv.y + 1);
    uint lattice_index = indexOfLattice(streaming_uv);
    float rho = macro_info[indexOfFluid(streaming_uv)].z;
    // for (int i = 0; i < 9; i++) {
    //   rho += streamingCells[indexOfLattice(streaming_uv) + i];
    // }
    streamingCells[lattice_index + 1] += rho * 0.1 / 9.0;
    streamingCells[lattice_index + 4] = streamingCells[lattice_index + 2];
    streamingCells[lattice_index + 7] =
        streamingCells[lattice_index + 5] - rho * 0.1 / 6.0;
    streamingCells[lattice_index + 8] =
        streamingCells[lattice_index + 6] + rho * 0.1 / 6.0;

  } else if (isBounceBackCell(material)) {
    // find lattice that direction quantities flowed in
    // bounce back the direction quantities to that lattice
    for (int i = 0; i < 9; i++) {
      // lattice coords that will bounce back to
      ivec2 streaming_uv = uv + ivec2(e(bounceBackDirection[i]));
      if (streaming_uv.x >= 0 && streaming_uv.x < int(lattice_num.x) &&
          streaming_uv.y >= 0 && streaming_uv.y < int(lattice_num.y)) {
        streamingCells[indexOfLattice(streaming_uv) + bounceBackDirection[i]] =
            collidingCells[indexOfLattice(streaming_uv) + i];
      }
    }
  }
}