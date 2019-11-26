// Calculate the divergence of the advected velocity field, and multiply by
// (2 * epsilon * rho / deltaT).
var calcDivergence = (function () {
    var shader = new gl.Shader(standardVertexShaderSrc, '\
      uniform float deltaT;         // Time between steps \n\
      uniform float rho;            // Density \n\
      uniform float epsilon;        // Distance between grid units \n\
      uniform sampler2D velocity;   // Advected velocity field, u_a \n\
      \
      varying vec2 textureCoord; \
      \
      vec2 u(vec2 coord) { \
        return texture2D(velocity, fract(coord)).xy; \
      } \
      \
      void main() { \
        gl_FragColor = vec4((-2.0 * epsilon * rho / deltaT) * ( \
          (u(textureCoord + vec2(epsilon, 0)).x - \
           u(textureCoord - vec2(epsilon, 0)).x) \
          + \
          (u(textureCoord + vec2(0, epsilon)).y - \
           u(textureCoord - vec2(0, epsilon)).y) \
        ), 0.0, 0.0, 1.0); \
      } \
    ');

    // Perform a single iteration of the Jacobi method in order to solve for
    // pressure.
    var jacobiIterationForPressure = (function () {
        var shader = new gl.Shader(standardVertexShaderSrc, '\
      uniform float epsilon;        // Distance between grid units \n\
      uniform sampler2D divergence; // Divergence field of advected velocity, d \n\
      uniform sampler2D pressure;   // Pressure field from previous iteration, p^(k-1) \n\
      \
      varying vec2 textureCoord; \
      \
      float d(vec2 coord) { \
        return texture2D(divergence, fract(coord)).x; \
      } \
      \
      float p(vec2 coord) { \
        return texture2D(pressure, fract(coord)).x; \
      } \
      \
      void main() { \
        gl_FragColor = vec4(0.25 * ( \
          d(textureCoord) \
          + p(textureCoord + vec2(2.0 * epsilon, 0.0)) \
          + p(textureCoord - vec2(2.0 * epsilon, 0.0)) \
          + p(textureCoord + vec2(0.0, 2.0 * epsilon)) \
          + p(textureCoord - vec2(0.0, 2.0 * epsilon)) \
        ), 0.0, 0.0, 1.0); \
      } \
    ');

        // Subtract the pressure gradient times a constant from the advected velocity
        // field.
var subtractPressureGradient = (function () {
            var shader = new gl.Shader(standardVertexShaderSrc, '\
      uniform float deltaT;         // Time between steps \n\
      uniform float rho;            // Density \n\
      uniform float epsilon;        // Distance between grid units \n\
      uniform sampler2D velocity;   // Advected velocity field, u_a \n\
      uniform sampler2D pressure;   // Solved pressure field \n\
      \
      varying vec2 textureCoord; \
      \
      float p(vec2 coord) { \
        return texture2D(pressure, fract(coord)).x; \
      } \
      \
      void main() { \
        vec2 u_a = texture2D(velocity, textureCoord).xy; \
        \
        float diff_p_x = (p(textureCoord + vec2(epsilon, 0.0)) - \
                          p(textureCoord - vec2(epsilon, 0.0))); \
        float u_x = u_a.x - deltaT/(2.0 * rho * epsilon) * diff_p_x; \
        \
        float diff_p_y = (p(textureCoord + vec2(0.0, epsilon)) - \
                          p(textureCoord - vec2(0.0, epsilon))); \
        float u_y = u_a.y - deltaT/(2.0 * rho * epsilon) * diff_p_y; \
        \
        gl_FragColor = vec4(u_x, u_y, 0.0, 0.0); \
      } \
    ');