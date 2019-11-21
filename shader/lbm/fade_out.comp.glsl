layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0) uniform ParticleUniform {
  // size of the lattice in the normalized coordinate space
  vec2 lattice_size;
  vec2 lattice_num;
  vec2 particle_num;
  // canvas pixel size
  vec2 canvas_size;
  // the value corresponding to one pixel in the normalized coordinate
  // space
  vec2 pixel_distance;
};

struct PixelInfo {
  float alpha;
  // absolute velocity
  float speed;
  // density
  float rho;
};
layout(set = 0, binding = 1) buffer Canvas { PixelInfo pixel_info[]; };

void main(void) {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  ivec2 size = ivec2(canvas_size);
  if (uv.x >= size.x || uv.y >= size.y) {
    return;
  }

  float alpha = pixel_info[uv.x + size.x * uv.y].alpha;
  if (alpha >= 0.5) {
    alpha *= 0.95;
  } else {
    alpha *= 0.5;
  }

  pixel_info[uv.x + size.x * uv.y].alpha = alpha;
}
