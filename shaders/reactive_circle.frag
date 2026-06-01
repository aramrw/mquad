#version 100
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
varying vec2 uv;

void main() {
  // Get audio data (0.0 to 1.0)
  float audio = texture2D(AudioTexture, vec2(uv.x, 0.5)).r;
  
  // Standardize coordinates (-1 to 1, aspect corrected)
  vec2 p = (uv.xy * 2.0 - 1.0);
  p.x *= Resolution.x / Resolution.y;

  // Music-reactive distortion for the background colors
  vec3 color = vec3(
      sin(p.x * 4.0 + Time + audio * 5.0),
      sin(p.y * 4.0 - Time + audio * 5.0),
      0.6 + (audio * 0.4)
    );

  // A pulsating ring in the center that jumps to the beat
  float pulse = audio * 0.6;
  float dist = length(p);
  float ring = smoothstep(0.1, 0.0, abs(dist - (0.4 + pulse)));
  
  // Add cyan glow to the ring
  color += ring * vec3(0.0, 1.0, 0.9);

  // Add a subtle scanline effect reacting to volume
  color *= 0.8 + 0.2 * sin(uv.y * 100.0 + Time * 10.0 + audio * 20.0);

  gl_FragColor = vec4(color, 1.0);
}
