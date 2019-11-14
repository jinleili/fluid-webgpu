layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0) uniform FieldUniform
{
    vec2 canvas_size;
    ivec2 particle_size;
    ivec4 field_size;
};
layout(binding = 1, rgba8) uniform image2D canvas;

void main(void)
{
    ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 info = ivec2(canvas_size);
    if (uv.x >= info.x || uv.y >= info.y) {
        return;
    }

    vec4 color = imageLoad(canvas, uv);
    if (color.a >= 0.3) {
        color.a *= 0.95;
    } else {
        color.a *= (1.0 - color.a) * 0.1;
    }
    imageStore(canvas, uv, color);
}
