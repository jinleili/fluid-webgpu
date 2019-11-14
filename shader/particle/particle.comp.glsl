layout(local_size_x = 1, local_size_y = 1) in;

layout(set = 0, binding = 0) uniform FieldUniform
{
    vec2 canvas_size;
    ivec2 particle_size;
    ivec4 field_size;
};

layout(set = 0, binding = 1) buffer ParticleBuffer { float pb[]; };
layout(set = 0, binding = 2) buffer FieldBuffer { vec2 fb[]; };

bool inside_world(vec2 v)
{
    return (v.x >= 0.0 && v.x <= canvas_size.x - 1.0) 
    && (v.y >= 0.0 && v.y <= canvas_size.y - 1.0);
}

vec2 invert_pos(vec2 p)
{
    float x = p.x;
    float y = p.y;
    if (x < 0.0)
        x = canvas_size.x - 2.0;
    else if (x > canvas_size.x - 1.0 )
    // 设置为边界值 0.0 会有问题，可能是由于浮点数不能准确的比较大小导致的
        x = 1.0;

    if (y < 0.0)
        y = canvas_size.y - 2.0;
    else if (y > canvas_size.y - 1.0)
        y = 1.0;

    return vec2(x, y);
}

void main() {
    ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
    // particle buffer index
    uint particle_index = (uv.y * particle_size.x + uv.x) * 3;
    vec3 pos = vec3(pb[particle_index], pb[particle_index + 1], pb[particle_index + 2]);
    // calculate the vector field grid index where the particle's current pos is located
    vec2 stride = canvas_size.xy / vec2(field_size.xy);
    uint field_index = uint(floor(pos.x / stride.x) + floor(pos.y / stride.y) * field_size.x);
    vec2 velocity = fb[field_index];
    float w = 0.0;
    if (!inside_world(pos.xy)) {
        pos.xy = invert_pos(pos.xy);
    } 
    pos.xy += velocity;
    
    pb[particle_index] = pos.x;
    pb[particle_index + 1] = pos.y;
    pb[particle_index + 2] = w;
}