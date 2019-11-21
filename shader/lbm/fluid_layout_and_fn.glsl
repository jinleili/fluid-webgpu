// D2Q9 流体相关的定义及函数

layout(set = 0, binding = 0) uniform FluidUniform
{
    // e 表示D2Q9离散速度模型速度空间的速度配置
    // w 表示每个方向上的权重
    vec4 e_and_w[9];
    // lattice 在正规化坐标空间的大小
    vec2 lattice_size;
    vec2 lattice_num;
    vec2 particle_num;
    // 正规化坐标空间里，一个像素对应的距离值
    // xcode metal 参数验证时报错：validateComputeFunctionArguments:852: failed
    // assertion `Compute Function(main0): argument v_26[0] from buffer(0) with
    // offset(0) and length(172) has space for 172 bytes, but argument has a
    // length(176).'
    vec2 pixel_distance;
};

struct Particle {
    vec2 pos;
    // 初始位置,用于重置位置
    vec2 pos_initial;
    float life_time;
    // 淡出值:[1.0, 0.0]
    float fade;
};

// 为了获得比较好的模拟效果，马赫数需要确保比较小
// 为宏观流速，为了减小误差，特征流速需要设定成比较小的值，最好不要超过 0.1
// Cs 表示声速
const float Cs2 = 1.0 / 3.0;
// τ represents the viscosity of the fluid, given by τ = 0.5 * (1.0 + 6niu )
const float tau = 0.8;
//弛豫时间
const float omega = 1.0 / tau;

// 获取离散速度模型速度空间的速度配置
vec2 e(int direction)
{
    return e_and_w[direction].xy;
}

// 获取某个方向上的权重
float w(int direction)
{
    return e_and_w[direction].z;
}

float equilibrium(vec2 velocity, float rho, int direction, float usqr)
{
    float e_dot_u = dot(e(direction), velocity);

    // pow(x, y) 要求 x 参数不能为负，e_dot_u
    // 是有可能为负的，所以求它的平方不能用  pow 内置函数 当 i == 0 时，feq =
    // rho * weight[i] * (1.0 - 1.5 * dot(velocity, velocity) / Cs2);
    // return rho * w(i) * (1.0 + 3.0 * e_dot_u / Cs2 + 4.5 * (e_dot_u * e_dot_u) / pow(Cs2, 2.0) - 1.5 * dot(velocity, velocity) / Cs2);
    return rho * w(direction) * (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}

int indexOfLattice(ivec2 uv)
{
    return (uv.x + (uv.y * int(lattice_num.x))) * 9;
}

int indexOfFluid(ivec2 uv)
{
    return uv.x + (uv.y * int(lattice_num.x));
}

int indexOfParticle(ivec2 uv)
{
    return (uv.x + (uv.y * int(particle_num.x)));
}

bool isBounceBackCell(int material) {
    return material == 2;
}

bool isBulkFluidCell(int material)
{
    return material == 1 || material == 5 || material == 6;
}

// 流入区
bool isInflowCell(int material)
{
    return material == 5;
}

// 流出区
bool isOutflowCell(int material)
{
    return material == 6;
}