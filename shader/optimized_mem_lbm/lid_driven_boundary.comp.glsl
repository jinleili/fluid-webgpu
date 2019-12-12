layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  int material = int(lattice_info[fieldIndex(uv)].material);
  if (isLidDrivenCell(material)) {
    uint target_index = latticeIndex(uvec2(uv.x, uv.y + 1));
    uint cur_index = latticeIndex(uv);
    float rho = macro_info[fieldIndex(uv) * 3 + 2];

    collid_streaming_cells[target_index + 1] += rho * 0.1 / 9.0;
    // collid_streaming_cells[target_index + 4] =
    //     collid_streaming_cells[cur_index + 2];
    collid_streaming_cells[target_index + 7] =
        collid_streaming_cells[cur_index + 5] - rho * 0.1 / 6.0;
    collid_streaming_cells[target_index + 8] =
        collid_streaming_cells[cur_index + 6] + rho * 0.1 / 6.0;
  }

// on-grid bounce back
#include "optimized_mem_lbm/code_block/cal_on_grid_bb.glsl"
}