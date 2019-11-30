
layout(set = 0, binding = 0) uniform D2Q9Uniform {
  // e: D2Q9 model direction coordinate
  // w: direction's weight
  vec4 e_and_w[9];
};

layout(set = 0, binding = 1) uniform FluidUniform {
  // size of the lattice in the normalized coordinate space
  vec2 lattice_size;
  vec2 lattice_num;
  vec2 particle_num;

  // one pixel in the normalized coordinate space
  //
  // xcode metal validation error：validateComputeFunctionArguments:852:
  // failed assertion `Compute Function(main0): argument v_26[0] from
  // buffer(0) with offset(0) and length(172) has space for 172 bytes, but
  // argument has a length(176).'
  vec2 pixel_distance;
  // τ represents the viscosity of the fluid, given by τ = 0.5 * (1.0 + 6niu )
  vec2 tau_and_omega;
};

layout(set = 0, binding = 2) buffer FluidBuffer0 { float collidingCells[]; };
layout(set = 0, binding = 3) buffer FluidBuffer1 { float streamingCells[]; };

layout(set = 0, binding = 4) buffer FluidBuffer2 {
  // macro_info.rg is macroscope velocity
  // macro_info.b is macroscope dencity
  vec4 macro_info[];
};

// sound speed
const float Cs2 = 1.0 / 3.0;
const int REVERSED_DERECTION[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);

// direction's coordinate
vec2 e(int direction) { return e_and_w[direction].xy; }
// direction's weight
float w(int direction) { return e_and_w[direction].z; }

float tau() { return tau_and_omega.x; }

float omega() { return tau_and_omega.y; }

float equilibrium(vec2 velocity, float rho, int direction, float usqr) {
  float e_dot_u = dot(e(direction), velocity);
  // internal fn pow(x, y) requires x cannot be negative
  return rho * w(direction) *
         (1.0 + 3.0 * e_dot_u + 4.5 * (e_dot_u * e_dot_u) - usqr);
}

int latticeIndex(ivec2 uv) { return (uv.x + (uv.y * int(lattice_num.x))) * 9; }
int fieldIndex(ivec2 uv) { return uv.x + (uv.y * int(lattice_num.x)); }
int particleIndex(ivec2 uv) { return (uv.x + (uv.y * int(particle_num.x))); }

bool isBounceBackCell(int material) { return material == 2; }
bool isLidDrivenCell(int material) { return material == 3; }

bool isBulkFluidCell(int material) {
  return material == 1 || material == 5 || material == 6;
}

// inflow area
bool isInflowCell(int material) { return material == 5; }

// outflow area
bool isOutflowCell(int material) { return material == 6; }