#version 150
precision lowp float;
in vec2 uv;
out vec4 fragColor;
uniform sampler2D AudioTexture;

uniform vec2 u_resolution;
uniform float Time;

void main() {
  // Center and normalize coordinates
  vec2 uv = (gl_FragCoord.xy - 0.5
        * u_resolution.xy)
      / u_resolution.y;

  float speed = Time * 0.5;

  // abs() mirrors the line across the axis; smoothstep hardens the edge
  float line = smoothstep(0.02, 0.01, abs(uv.y + sin(speed) * 0.4));

  fragColor = vec4(vec3(line), 1.0);
}
