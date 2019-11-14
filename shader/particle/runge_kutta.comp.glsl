layout(local_size_x = 1, local_size_y = 1) in;

layout(set = 0, binding = 0) uniform ParticleUniform
{
    vec4 world;
};
layout(set = 0, binding = 1) buffer ParticleBuffer { vec3 data[]; };

vec2 f(vec2 v) {
    // return vec2(cos(v.x * sin(v.y)), sin(v.x - v.y));

    // change this to get a new vector field
    return vec2(0.1 * v.y, -0.2 * v.y);
}

vec2 classical_runge_kutta(float h, vec2 v)
{
    vec2 k1 = f(v);
    vec2 k2 = f(v + h / 2. * k1);
    vec2 k3 = f(v + h / 2. * k2);
    vec2 k4 = f(v + h * k3);

    return v + h * (1. / 6. * k1 + 1. / 3. * k2 + 1. / 3. * k3 + 1. / 6. * k4);
}

// pseudo random numbers for particle placement
float rand(vec2 v)
{
    return fract(sin(dot(v, vec2(12.9898, 78.233))) * 43758.5453);
}

float mapUnitToWorldX(float s)
{
    return -(world.x) + s * world.x;
}

float mapUnitToWorldY(float s)
{
    return -1.0 + s * 2.0;
}

bool inside_world(vec2 v)
{
    return v.x > -1.0 
        && v.x < 1.0 
        && v.y > -1.0 
        && v.y < 1.0;
}

void main()
{
    uint i = gl_GlobalInvocationID.x;
    vec3 buf = data[i];
    vec2 v = buf.xy;
    vec2 w = classical_runge_kutta(0.016, v);

    if (inside_world(v)) {
        data[i] = vec3(w.x, w.y, buf.z + 0.016);
    } else {
        data[i] = vec3(mapUnitToWorldX(rand(v)), mapUnitToWorldY(rand(w)), rand(v + w) * 5.);
    }
}