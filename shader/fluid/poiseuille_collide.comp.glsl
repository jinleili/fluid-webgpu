layout(local_size_x = 16, local_size_y = 16) in;

#include "fluid/fluid_layout_and_fn.glsl"

layout(set = 0, binding = 1) buffer FluidBuffer0 { float collidingCells[]; };
layout(set = 0, binding = 2) buffer FluidBuffer1 { float streamingCells[]; };
// rgb 表示对应 lattice 上的宏观速度密度
layout(set = 0, binding = 3) buffer FluidBuffer2 { vec4 macro_info[]; };

// 回弹方向对应的传播索引
const int bounceBackDirection[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

const vec2 force = vec2(0.01, 0.0);

// collide
void updateCollide(ivec2 uv, int direction, float collide) {
  // 避免出现 NaN， inf, -inf, 将值限定在一个范围
  collidingCells[indexOfLattice(uv) + direction] =
      clamp(collide, -100.0, 100.0);
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
  // 边界节点不需要计算碰撞
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
    // U = sum_fi*ei / rho
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
  // updateMacro2(uv);

  // Collision step: fout = fin - omega * (fin - feq)
  // 平衡方程最后一项：1.5 * 速度绝对值的平方
  float usqr = 1.5 * (velocity.x * velocity.x + velocity.y * velocity.y);
  for (int i = 0; i < 9; i++) {
    // 碰撞
    // float collide = f_i[i] + omega * (equilibrium(velocity, rho, i, usqr) -
    // f_i[i]);
    float collide =
        f_i[i] - omega * (f_i[i] - equilibrium(velocity, rho, i, usqr));

    updateCollide(uv, i, collide);
  }
}
