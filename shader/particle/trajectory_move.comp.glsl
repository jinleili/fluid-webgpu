layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0) uniform ParticleUniform {
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

layout(set = 0, binding = 1) uniform AnimateUniform {
  //
  float life_time;
  float fade_out_factor;
  float speed_factor;
};

struct TrajectoryParticle {
  vec2 pos;
  // initial position, use to reset particle position
  vec2 pos_initial;
  float life_time;
  // alpha value:[1.0, 0.0]
  float fade;
};

layout(set = 0, binding = 2) buffer ParticleBuffer { TrajectoryParticle pb[]; };
layout(set = 0, binding = 3) buffer FieldBuffer { vec3 fb[]; };

struct PixelInfo {
  float alpha;
  // absolute velocity
  float speed;
  // density
  float rho;
};
layout(set = 0, binding = 4) buffer Canvas { PixelInfo pixel_info[]; };

struct LatticeInfo {
  int material;
  //  dynamic iter value, change material ultimately
  float iter;
  float threshold;
};

layout(set = 0, binding = 5) buffer LatticeBuffer {
  LatticeInfo lattice_info[];
};

vec3 src_3f(int u, int v) {
  u = clamp(u, 0, int(lattice_num.x - 1));
  v = clamp(v, 0, int(lattice_num.y - 1));

  return fb[v * lattice_num.x + u];
}

uint fieldIndex(uvec2 uv) { return uv.x + (uv.y * lattice_num.x); }

#include "func/bilinear_interpolate_3f.glsl"

int particleIndex(ivec2 uv) { return (uv.x + (uv.y * int(particle_num.x))); }
bool isBounceBackCell(int material) { return material == 2; }

void update_canvas(int point_size, ivec2 canvas_size,
                   TrajectoryParticle particle, int px, int py, vec3 f_info) {
  PixelInfo info =
      PixelInfo(particle.fade, abs(f_info.x) + abs(f_info.y) * 100.0, f_info.z);
  for (uint x = 0; x < point_size; x++) {
    for (uint y = 0; y < point_size; y++) {
      ivec2 coords = ivec2(px + x, py + y);
      if (coords.x >= 0 && coords.x < canvas_size.x && coords.y >= 0 &&
          coords.y < canvas_size.y) {
        pixel_info[coords.x + canvas_size.x * coords.y] = info;
      }
    }
  }
}

void main() {
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(particle_num.x) || uv.y >= int(particle_num.y)) {
    return;
  }

  TrajectoryParticle particle = pb[particleIndex(uv)];
  if (particle.life_time <= 0.1) {
    particle.fade = 0.0;
    particle.pos = particle.pos_initial;
    particle.life_time = life_time;
  }

  if (particle.life_time > 0.1) {
    particle.life_time -= 1.0;

    // Calculate the lattice in which this particle is located
    // particle's coordinate space is【-1， 1】，need convert to 【0， 2】
    vec2 new_pos = particle.pos.xy + vec2(1.0, 1.0);
    vec2 ij = vec2((new_pos.x / lattice_size.x) - 0.5,
                   (new_pos.y / lattice_size.y) - 0.5);
    vec3 f_info = bilinear_interpolate_3f(ij);
    // vec3 f_info = src_3f(int(floor(ij.x)), int(floor(ij.y)));
    particle.pos.xy += (f_info.xy * pixel_distance * speed_factor);
    // fade in effect
    if (particle.fade < 1.0) {
      if (particle.fade < 0.9) {
        particle.fade += 0.1;
      } else {
        particle.fade = 1.0;
      }
    }

    // calculate if particle's new position is inside obstacle or boundary
    // lattice
    ivec2 lattice = ivec2((particle.pos.xy + vec2(1.0, 1.0)) / lattice_size);
    uint field_index = fieldIndex(uvec2(clamp(lattice, uvec2(0), (lattice_num - uvec2(1)))));
    int material = lattice_info[field_index].material;
    if (isBounceBackCell(material) == false) {
      // update pixel's alpha value：
      //
      // first, need calculate out pixel coordinate
      ivec2 pixel_coords =
          ivec2(round((particle.pos.x + 1.0) / pixel_distance.x),
                round((particle.pos.y + 1.0) / pixel_distance.y));
      int point_size = 2;
      int px = pixel_coords.x - point_size / 2;
      int py = pixel_coords.y - point_size / 2;
      // then, update values by scope
      update_canvas(point_size, ivec2(canvas_size), particle, px, py, f_info);
    }
  }

  pb[particleIndex(uv)] = particle;
}