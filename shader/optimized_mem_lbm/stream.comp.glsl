layout(local_size_x = 16, local_size_y = 16) in;

#include "optimized_mem_lbm/code_block/layout_and_fn.glsl"

layout(set = 1, binding = 0) uniform Q9DirectionUniform {
  uint direction;
  vec4 any[15];
};

void main() {
  uvec2 uv = uvec2(gl_GlobalInvocationID.xy);
  if (uv.x >= lattice_num.x || uv.y >= lattice_num.y) {
    return;
  }
  uint field_index = fieldIndex(uv);
  int material = int(lattice_info[field_index].material);
  // at boundary lattice, not need calculate collide and stream
  if (isBounceBackCell(material) || isLidDrivenCell(material)) {
    return;
  }

  ivec2 streaming_target = ivec2(uv + e(direction));
  if (streaming_target.x < 0) {
    streaming_target.x = int(lattice_num.x - 1);
  } else if (streaming_target.x >= lattice_num.x) {
    streaming_target.x = 0;
  }
  if (streaming_target.y < 0) {
    streaming_target.y = int(lattice_num.y - 1);
  } else if (streaming_target.y >= lattice_num.y) {
    streaming_target.y = 0;
  }

  collid_streaming_cells[latticeIndex(uvec2(streaming_target)) + direction] =
      temp_scalar_cells[field_index];
}
