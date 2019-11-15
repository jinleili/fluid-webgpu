layout(local_size_x = 16, local_size_y = 16) in;

#include "fluid/fluid_layout_and_fn.glsl"

layout(set = 0, binding = 1) buffer FluidBuffer0 { float collidingCells[]; };
layout(set = 0, binding = 2) buffer FluidBuffer1 { float streamingCells[]; };
// rgb 表示对应 lattice 上的宏观速度密度
layout(set = 0, binding = 3) buffer FluidBuffer2 { vec4 macro_info[]; };

// 回弹方向对应的传播索引
const int bounceBackDirection[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

// 普通回弹
// direction: 方向索引
void setNormalBounceBack(ivec2 uv, int direction) {
  streamingCells[indexOfLattice(uv) + direction] =
      streamingCells[indexOfLattice(uv) + bounceBackDirection[direction]];
  streamingCells[indexOfLattice(uv) + bounceBackDirection[direction]] = 0.0;
}

// 流出（迁移）：将当前格子的量迁移到周围格子
void streaming_out(ivec2 uv, int direction) {
  // https://pdfs.semanticscholar.org/e626/ca323a9a8a4ad82fb16ccbbbd93ba5aa98e0.pdf
  // 沿着当前方向量流向旁边格子上的同一方向量
  ivec2 new_uv = uv + ivec2(e(direction));
  streamingCells[indexOfLattice(new_uv) + direction] =
      collidingCells[indexOfLattice(uv) + direction];
}

// 流入（迁移）：将周围格子的量迁移到当前格子
void streaming_in(ivec2 uv, int direction) {
  ivec2 new_uv = uv - ivec2(e(direction));
  streamingCells[indexOfLattice(uv) + direction] =
      collidingCells[indexOfLattice(new_uv) + direction];
}

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  // 用来判断当前是不是边界，障碍等
  int material = int(macro_info[indexOfFluid(uv)].w);

  if (isBulkFluidCell(material)) {
    for (int i = 0; i < 9; i++) {
      streaming_in(uv, i);
    }
  }
  // 计算着色器里只能通过 if 来处理分支条件，不能通过 else 来做分支条件
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