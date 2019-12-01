
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

struct PixelInfo {
  float alpha;
  // absolute velocity
  float speed;
  // density
  float rho;
};
layout(set = 0, binding = 2) buffer Canvas { PixelInfo pixel_info[]; };

void main(void) {
  ivec2 pixel_coord = ivec2(round(gl_FragCoord.x), round(gl_FragCoord.y));
  pixel_coord.x = min(pixel_coord.x, int(canvas_size.x - 1.0));
  pixel_coord.y = min(pixel_coord.y, int(canvas_size.y - 1.0));

  PixelInfo pixel =
      pixel_info[pixel_coord.x + pixel_coord.y * int(canvas_size.x)];

  frag_color = vec4(1.0, 1.0, 1.0, pixel.alpha);
}