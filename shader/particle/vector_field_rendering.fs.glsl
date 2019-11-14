
layout(location = 0) out vec4 frag_color;
layout(location = 0) in vec4 point_color;

void main(void) {
    frag_color = point_color;
}