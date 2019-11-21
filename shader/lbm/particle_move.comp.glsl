layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0) uniform ParticleUniform {
  // size of the lattice in the normalized coordinate space
  vec2 lattice_size;
  vec2 lattice_num;
  vec2 particle_num;
  // canvas pixel size
  vec2 canvas_size;
  // the value corresponding to one pixel in the normalized coordinate
  // space
  vec2 pixel_distance;
};

struct Particle {
  vec2 pos;
  // 初始位置,用于重置位置
  vec2 pos_initial;
  float life_time;
  // 淡出值:[1.0, 0.0]
  float fade;
};
layout(set = 0, binding = 1) buffer ParticleBuffer { Particle pb[]; };
layout(set = 0, binding = 2) buffer FieldBuffer { vec4 fb[]; };

struct PixelInfo {
  float alpha;
  // absolute velocity
  float speed;
  // density
  float rho;
};
layout(set = 0, binding = 3) buffer Canvas { PixelInfo pixel_info[]; };

vec4 srcData(int u, int v) { return fb[v * int(lattice_num.x) + u]; }

vec4 bilinear_interpolate(vec2 uv) {
  int minX = int(floor(uv.x));
  int minY = int(floor(uv.y));
  int valid_min_x = minX < 0 ? 0 : minX;
  int valid_min_y = minY < 0 ? 0 : minY;
  int min_plus1_x =
      minX + 1 > int(lattice_num.x) ? int(lattice_num.x) : minX + 1;
  int min_plus1_y =
      minY + 1 > int(lattice_num.y) ? int(lattice_num.y) : minY + 1;

  float fx = uv.x - float(minX);
  float fy = uv.y - float(minY);
  // 插值公式： f(i+u,j+v) = (1-u)(1-v)f(i,j) + (1-u)vf(i,j+1) +
  // u(1-v)f(i+1,j) + uvf(i+1,j+1)
  return srcData(valid_min_x, valid_min_y) * ((1.0 - fx) * (1.0 - fy)) +
         srcData(valid_min_x, min_plus1_y) * ((1.0 - fx) * fy) +
         srcData(min_plus1_x, valid_min_y) * (fx * (1.0 - fy)) +
         srcData(min_plus1_x, min_plus1_y) * (fx * fy);
}

int indexOfParticle(ivec2 uv) { return (uv.x + (uv.y * int(particle_num.x))); }
bool isBounceBackCell(int material) { return material == 2; }

void update_canvas(int point_size, ivec2 canvas_size, Particle particle, int px,
                   int py, vec4 f_info) {
  PixelInfo info =
      PixelInfo(particle.fade, abs(f_info.x) + abs(f_info.y) * 100.0, f_info.z);
  for (int x = 0; x < point_size; x++) {
    for (int y = 0; y < point_size; y++) {
      ivec2 coords = ivec2(px + x, py + y);
      if (coords.x >= 0 && coords.x < canvas_size.x && coords.y >= 0 &&
          coords.y < canvas_size.y) {
        pixel_info[coords.x + canvas_size.x * coords.y] = info;
      }
    }
  }
}

void main() {
  // 粒子个数与格子是不一致的
  ivec2 uv = ivec2(gl_GlobalInvocationID.xy);
  if (uv.x >= int(particle_num.x) || uv.y >= int(particle_num.y)) {
    return;
  }

  Particle particle = pb[indexOfParticle(uv)];
  // 判断粒子的生命阶段
  if (particle.life_time <= 0.1) {
    particle.fade = 0.0;
    particle.pos = particle.pos_initial;
    particle.life_time = 60.0;
  }

  if (particle.life_time > 0.1) {
    particle.life_time -= 1.0;

    // 计算粒子所在的 lattice
    // 粒子的坐标空间是【-1， 1】，需要转到 【0， 2】
    vec2 new_pos = particle.pos.xy + vec2(1.0, 1.0);
    vec2 ij = vec2(new_pos.x / lattice_size.x - 0.5,
                   new_pos.y / lattice_size.y - 0.5);
    vec4 f_info = bilinear_interpolate(ij);
    // vec4 f_info = srcData(int(floor(ij.x)), int(floor(ij.y)));

    particle.pos.xy += (f_info.xy * pixel_distance * 20.0);
    // 淡入效果
    if (particle.fade < 1.0) {
      if (particle.fade < 0.9) {
        particle.fade += 0.1;
      } else {
        particle.fade = 1.0;
      }
    }

    // 计算新位置是否在边界或障碍格子内
    ivec2 lattice = ivec2((particle.pos.xy + vec2(1.0, 1.0)) / lattice_size);
    int material = int(srcData(lattice.x, lattice.y).w);
    if (isBounceBackCell(material) == false) {
      // 更新画布上对应像素的 alpha 值：
      // 先计算出对应到画面上的像素坐标;
      ivec2 pixel_coords =
          ivec2(round((particle.pos.x + 1.0) / pixel_distance.x),
                round((particle.pos.y + 1.0) / pixel_distance.y));
      int point_size = 2;
      int px = pixel_coords.x - point_size / 2;
      int py = pixel_coords.y - point_size / 2;

      // 更新指定范围内的所有像素的 alpha
      update_canvas(point_size, ivec2(canvas_size), particle, px, py, f_info);
    }
  }

  pb[indexOfParticle(uv)] = particle;
}