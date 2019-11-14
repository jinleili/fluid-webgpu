layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0) uniform ParticleUniform {
  // lattice 在正规化坐标空间的大小
  vec2 lattice_size;
  vec2 lattice_num;
  vec2 particle_num;
  // 画布像素尺寸
  vec2 canvas_size;
  // 正规化坐标空间里，一个像素对应的距离值
  vec2 pixel_distance;
};

layout(set = 0, binding = 1) buffer Canvas { float pixel_alpha[]; };

void main(void) {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  ivec2 size = ivec2(canvas_size);
  if (uv.x >= size.x || uv.y >= size.y) {
    return;
  }

  float alpha = pixel_alpha[uv.x + size.x * uv.y];
  if (alpha >= 0.05) {
    alpha *= 0.9;
  } else {
    alpha *= 0.5;
  }

  pixel_alpha[uv.x + size.x * uv.y] = alpha;
}
