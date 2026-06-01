#version 100
precision mediump float;

uniform vec2 Resolution;
varying vec2 uv;
uniform float Time;

void main() {
  vec2 p = (uv.xy * 1.0 - Resolution.xy) / min(Resolution.x, Resolution.y);

  vec3 color = vec3(
      sin(p.x),
      sin(p.y),
      0.8
    );

  gl_FragColor = vec4(color, 1.0);
}
