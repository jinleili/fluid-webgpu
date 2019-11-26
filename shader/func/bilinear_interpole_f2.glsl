vec2 bilinear_interpolate_2f(vec2 uv) {
  int minX = int(floor(uv.x));
  int minY = int(floor(uv.y));

  float fx = uv.x - float(minX);
  float fy = uv.y - float(minY);
  // formula： f(i+u,j+v) = (1-u)(1-v)f(i,j) + (1-u)vf(i,j+1) +
  // u(1-v)f(i+1,j) + uvf(i+1,j+1)
  return src_2f(minX, minY) * ((1.0 - fx) * (1.0 - fy)) +
         src_2f(minX, minY + 1) * ((1.0 - fx) * fy) +
         src_2f(minX + 1, minY) * (fx * (1.0 - fy)) +
         src_2f(minX + 1, minY + 1) * (fx * fy);
}