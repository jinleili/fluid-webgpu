layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) buffer FluidBuffer { vec4 fb[]; };
layout(set = 0, binding = 1) buffer ScalarBuffer { float diffuse[]; };

void main() {
    
}