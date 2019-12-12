layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"
layout(set = 0, binding = 6) buffer DiffuseBuffer0 { float diffuse_cells[]; };

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  int material = int(lattice_info[fieldIndex(uv)].material);
  // on-grid bounce back
  if (isBounceBackCell(material)) {
    for (uint i = 0; i < 9; i++) {
      ivec2 streaming_uv = ivec2(uv + e(REVERSED_DERECTION[i]));
      if (streaming_uv.x >= 0 && streaming_uv.x < lattice_num.x &&
          streaming_uv.y >= 0 && streaming_uv.y < lattice_num.y) {
        diffuse_cells[latticeIndex(streaming_uv) + REVERSED_DERECTION[i]] =
            diffuse_cells[latticeIndex(uv) + i];
        diffuse_cells[latticeIndex(uv) + i] = 0.0;
      }
    }
  }
}