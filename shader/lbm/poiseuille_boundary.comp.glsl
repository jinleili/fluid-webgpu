layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

layout(set = 0, binding = 1) buffer FluidBuffer0 { float collidingCells[]; };
layout(set = 0, binding = 2) buffer FluidBuffer1 { float streamingCells[]; };
// rgb 表示对应 lattice 上的宏观速度密度
layout(set = 0, binding = 3) buffer FluidBuffer2 { vec4 macro_info[]; };

const int bounceBackDirection[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  int material = int(macro_info[indexOfFluid(uv)].w);

  if (isBounceBackCell(material)) {
    // 边界格子，直接找到入流格子，将其量回弹回去
    // 要回弹的方向的量是反方向格子上的同一方向量流出过来 
    for (int i = 0; i < 9; i++) {
      // 反方向上的格子坐标
      ivec2 streaming_uv = uv + ivec2(e(bounceBackDirection[i]));
      if (streaming_uv.x >= 0 && streaming_uv.x < int(lattice_num.x) &&
          streaming_uv.y >= 0 && streaming_uv.y < int(lattice_num.y)) {
        streamingCells[indexOfLattice(streaming_uv) + bounceBackDirection[i]] =
            collidingCells[indexOfLattice(streaming_uv) + i];
      }
    }
  }
}