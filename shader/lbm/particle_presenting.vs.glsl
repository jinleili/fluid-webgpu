layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoord;

layout(set = 0, binding = 0) uniform MVPUniform { mat4 mvp_matrix; };

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

layout(location = 0) out vec2 uv;

void main() {
  gl_Position = mvp_matrix * vec4(position, 1.0);
  uv = texcoord;
}
