layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 1) uniform texture2D skin_texture;

void main(void) {
  //   frag_color = imageLoad(skin_texture, ivec2(uv));

  frag_color = vec4(0.1, 0.2, 0.3, 1.0);

}
