layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 1) uniform ParticleUniform {
  // size of the lattice in the normalized coordinate space
  vec2 lattice_size;
  uvec2 lattice_num;
  uvec2 particle_num;
  // canvas pixel size
  uvec2 canvas_size;
  // the value corresponding to one pixel in the normalized coordinate
  // space
  vec2 pixel_distance;
};
layout(set = 0, binding = 2) buffer FluidBuffer { float fb[]; };
layout(set = 0, binding = 3) buffer ScalarBuffer { float diffuse[]; };

float src_1f(int u, int v) {
  u = clamp(u, 0, int(lattice_num.x - 1));
  v = clamp(v, 0, int(lattice_num.y - 1));

  return diffuse[v * lattice_num.x + u];
}

#include "func/bilinear_interpolate_1f.glsl"

void main() {
  vec2 pos = uv + vec2(1.0, 1.0);
  vec2 ij = (pos / lattice_size) - vec2(0.5);
  float concentration = bilinear_interpolate_1f(ij);
  float factor = 1.0;
  if (concentration < 1.0) {
    factor = (3.0 - concentration * 2.0);
  }
  concentration = clamp(concentration * factor, 0.0, 1.0);

  frag_color = vec4(vec3(1.0 - concentration), 1.0);
  // frag_color = vec4(0.5);
}