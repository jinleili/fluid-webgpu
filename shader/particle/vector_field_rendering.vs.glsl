
layout(location = 0) in int particle_index;
layout(location = 0) out vec4 point_color;

layout(set = 0, binding = 0) uniform FieldUniform
{
    vec2 canvas_size;
    ivec2 particle_size;
    ivec4 field_size;
};
layout(set = 0, binding = 1) buffer ParticleBuffer { float pb[]; };

void main() {
    // pixel coords convert to normal coords ([-1, 1])
    vec2 normal_coords = vec2(-1.0) + (vec2(pb[particle_index * 3], pb[particle_index * 3 + 1]) / canvas_size) * 2.0;

    point_color = vec4(0.9, 0.7, 0.9, 1.0);
 
    gl_Position = vec4(normal_coords, 0.0, 1.0);
    gl_PointSize = 2.0;
}