
layout(set = 0, binding = 0) uniform D2Q9Uniform {
  // e: D2Q9 model direction coordinate
  // w: direction's weight
  vec4 e_and_w[9];
};

layout(set = 0, binding = 1) uniform FluidUniform {
  // size of the lattice in the normalized coordinate space
  vec2 lattice_size;
  uvec2 lattice_num;
  uvec2 particle_num;
  // one pixel in the normalized coordinate space
  vec2 pixel_distance;
  // τ represents the viscosity of the fluid, given by τ = 0.5 * (1.0 + 6niu )
  vec2 tau_and_omega;
};

layout(set = 0, binding = 2) uniform TouchUniform {
  ivec2 touch_point;
  // left top lattice index, maybe out of screen
  ivec2 lt_lattice;
  uvec2 tex_size;
};

layout(set = 0, binding = 3) buffer LatticeBuffer {
  float collid_streaming_cells[];
};
// only temporarily save lattice one direction value
layout(set = 0, binding = 4) buffer TempScalarBuffer {
  float temp_scalar_cells[];
};

struct LatticeInfo {
  int material;
  int diffuse_step_count;
  //  dynamic iter value, change material ultimately
  float iter;
  float threshold;
};
layout(set = 0, binding = 5) buffer InfoBuffer { LatticeInfo lattice_info[]; };

struct MacroInfo {
  vec2 velocity;
  float rho;
};
layout(set = 0, binding = 6) buffer MacroBuffer { MacroInfo macro_info[]; };

layout(set = 0, binding = 7) buffer DiffuseBuffer { float diffuse[]; };

// sound speed
const float Cs2 = 1.0 / 3.0;
const int REVERSED_DERECTION[9] = int[](0, 3, 4, 1, 2, 7, 8, 5, 6);
const int INFLOW_DERECTION[9] = int[](0, 1, 2, 0, 4, 5, 0, 0, 8);
const int OUTFLOW_DERECTION[9] = int[](0, 0, 2, 3, 4, 0, 6, 7, 0);

// direction's coordinate
vec2 e(uint direction) { return e_and_w[direction].xy; }
// direction's weight
float w(uint direction) { return e_and_w[direction].z; }

float tau() { return tau_and_omega.x; }
float omega() { return tau_and_omega.y; }

uint latticeIndex(uvec2 uv) { return (uv.x + (uv.y * lattice_num.x)) * 9; }
uint fieldIndex(uvec2 uv) { return uv.x + (uv.y * lattice_num.x); }
uint particleIndex(uvec2 uv) { return (uv.x + (uv.y * particle_num.x)); }

bool isBounceBackCell(int material) { return material == 2; }
// block ink diffuse
bool isBlockCell(int material) { return material == 7; }

bool isBulkFluidCell(int material) {
  return material == 1 || material == 5 || material == 6;
}
