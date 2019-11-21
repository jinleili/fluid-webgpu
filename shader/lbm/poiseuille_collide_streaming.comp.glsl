layout(local_size_x = 16, local_size_y = 16) in;

#include "lbm/fluid_layout_and_fn.glsl"

layout(set = 0, binding = 1) buffer FluidBuffer0 { float collidingCells[]; };
layout(set = 0, binding = 2) buffer FluidBuffer1 { float streamingCells[]; };
// rgb 表示对应 lattice 上的宏观速度密度
layout(set = 0, binding = 3) buffer FluidBuffer2 { vec4 macro_info[]; };

// collide
void updateCollide(ivec2 uv, int direction, float collide) {
  // 避免出现 NaN， inf, -inf, 将值限定在一个范围
  collidingCells[indexOfLattice(uv) + direction] = collide;
}

// 流出（迁移）：将当前格子的量迁移到周围格子
void streaming_out(ivec2 uv, int direction, float collide) {
  // https://pdfs.semanticscholar.org/e626/ca323a9a8a4ad82fb16ccbbbd93ba5aa98e0.pdf
  // 沿着当前方向量流向旁边格子上的同一方向量
  ivec2 new_uv = uv + ivec2(e(direction));
  streamingCells[indexOfLattice(new_uv) + direction] = collide;
}
// 流入（迁移）：将周围格子的量迁移到当前格子
void streaming_in(ivec2 uv, int direction) {
  ivec2 new_uv = uv - ivec2(e(direction));
  streamingCells[indexOfLattice(uv) + direction] =
      collidingCells[indexOfLattice(new_uv) + direction];
}

// 更新流体宏观速度等信息
void updateMacro(ivec2 uv, vec2 velocity, float rho) {
  int destIndex = uv.x + uv.y * int(lattice_num.x);
  macro_info[destIndex].xyz = vec3(velocity, rho);
}

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(lattice_num).x || uv.y >= int(lattice_num.y)) {
    return;
  }
  // 用来判断当前是不是边界，障碍等
  int material = int(macro_info[indexOfFluid(uv)].w);
  // 边界节点不需要计算碰撞及流出
  if (isBounceBackCell(material)) {
    return;
  }

  float f_i[9];
  //格子点的迁移过程
  vec2 velocity = vec2(0.0);
  float rho = 0.0;

  for (int i = 0; i < 9; i++) {
    f_i[i] = streamingCells[indexOfLattice(uv) + i];
    rho += f_i[i];
    velocity += e(i) * f_i[i];
  }
  velocity = velocity / rho;

  if (isOutflowCell(material)) {
    // 出流恢复密度
    rho = 1.0;
  }
  if (isInflowCell(material)) {
    // 入流加一个速度（外力项）
    velocity = vec2(0.1, 0.05);
  }
  // 更新宏观速度，密度
  updateMacro(uv, velocity, rho);

  // Collision step: fout = fin - omega * (fin - feq)
  // 平衡方程最后一项：1.5 * 速度绝对值的平方
  float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
  for (int i = 0; i < 9; i++) {
    // 碰撞
    float collide =
        f_i[i] - omega * (f_i[i] - equilibrium(velocity, rho, i, usqr));

    updateCollide(uv, i, collide);
    streaming_out(uv, i, collide);
  }
}
