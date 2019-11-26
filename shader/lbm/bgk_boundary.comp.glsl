layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

const int bounceBackDirection[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  int material = int(macro_info[indexOfFluid(uv)].w);

  if (isBounceBackCell(material)) {
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