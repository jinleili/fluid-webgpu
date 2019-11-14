layout(local_size_x = 1, local_size_y = 1) in;

layout(set = 0, binding = 0) uniform FieldUniform
{
    vec2 canvas_size;
    ivec2 particle_size;
    ivec4 field_size;
};
layout(set = 0, binding = 1) buffer FieldBuffer { vec2 data[]; };

void set_vector(vec2 v, ivec2 uv) {
    data[uv.y * field_size.x + uv.x] = v;
}

void main()
{
    ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
    vec2 v = vec2(0.0);
    if (field_size.z == 0) {
        if (mod(uv.y, 2) == 0) {
            v = vec2(1.0, 0.0);
        } else {
            v = vec2(-1.0, 0.0);
        }
    } else if (field_size.z == 1) {
        v.x = -2.0 * mod(uv.y, 2) + 1.0;
        v.y = -2.0 * mod(uv.x, 2) + 1.0;
    } else if (field_size.z == 2) {
        float y = float(uv.y) - float(field_size.y) / 2.0;
        v.x = 0.1 * y;
        v.y = -0.2 * y;
    }
    
    set_vector(v, uv);
}