layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/code_block/fluid_layout_and_fn.glsl"

// frome current lattice streaming to neighbour
void streaming_out(uvec2 uv, uint direction, float collide) {
  // https://pdfs.semanticscholar.org/e626/ca323a9a8a4ad82fb16ccbbbd93ba5aa98e0.pdf
  // along current direction streaming to neighbour lattice same direction
  ivec2 new_uv = ivec2(uv + e(direction));
  // if not detect coordinate's legality, on iOS will cause error:
  // Execution of the command buffer was aborted due to an error during
  // execution. Ignored (for causing prior/excessive GPU errors) (IOAF code 4)
  if (new_uv.x > 0 && new_uv.x < lattice_num.x) {
    streamingCells[latticeIndex(new_uv) + direction] = collide;
  }
}

// frome neighbour lattice streaming in current
void streaming_in(uvec2 uv, uint direction) {
  ivec2 new_uv = ivec2(uv - e(direction));
  streamingCells[latticeIndex(uv) + direction] =
      collidingCells[latticeIndex(new_uv) + direction];
}

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  int material = int(lattice_info[fieldIndex(uv)].material);

  if (isBulkFluidCell(material)) {
    for (uint i = 0; i < 9; i++) {
      float collide = collidingCells[latticeIndex(uv) + i];
      streaming_out(uv, i, collide);
    }
  }
#include "lbm/code_block/cal_on_grid_bb.glsl"
}