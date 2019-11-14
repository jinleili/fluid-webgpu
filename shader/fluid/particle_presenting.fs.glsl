
layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 1) uniform ParticleUniform {
  // lattice 在正规化坐标空间的大小
  vec2 lattice_size;
  vec2 lattice_num;
  vec2 particle_num;
  // 画布像素尺寸
  vec2 canvas_size;
  // 正规化坐标空间里，一个像素对应的距离值'
  vec2 pixel_distance;
};

layout(set = 0, binding = 2) buffer Canvas { float pixel_alpha[]; };

void main(void) {
  ivec2 pixel_coord = ivec2(round(gl_FragCoord.x), round(gl_FragCoord.y));
  float alpha = pixel_alpha[pixel_coord.x + pixel_coord.y * int(canvas_size.x)];

  frag_color = vec4(0.9, 0.7, 0.9, alpha);
}