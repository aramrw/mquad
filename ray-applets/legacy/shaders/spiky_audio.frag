#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

void main() {
  // 1. Standardize coordinates (-1 to 1, aspect corrected)
  vec2 p = (uv.xy * 2.0 - 1.0);
  p.x *= Resolution.x / Resolution.y;

  // 2. Polar coordinates
  float angle = atan(p.y, p.x);
  float dist = length(p);

  // 3. Map angle to [0, 1] for texture sampling
  float norm_angle = (angle + 3.14159) / 6.28318;

  // Sample Waveform (Y=0.25) and Frequency (Y=0.75)
  float wave = texture(AudioTexture, vec2(norm_angle, 0.25)).r;
  float spec = texture(AudioTexture, vec2(norm_angle, 0.75)).r;

  // 4. Create the 'Spiky' deformation
  // Base radius + waveform pulse + spectral spikes
  float base_radius = 0.35;
  float deformation = wave * 0.1 + spec * 0.4;
  float target_dist = base_radius + deformation;

  // 5. Draw the spiky ring with glow
  float ring_edge = abs(dist - target_dist);
  float glow = 0.015 / (ring_edge + 0.005);

  // 6. Multi-color palette shifting over time
  vec3 color = 0.5 + 0.5 * cos(Time + angle + vec3(0, 2, 4));

  // Add volume-reactive brightness
  vec3 final_color = color * glow * (0.8 + spec * 0.5);

  // 7. Electronic scanlines reacting to bass
  float scanline = 0.9 + 0.1 * sin(uv.y * 180.0 - Time * 5.0);
  final_color *= scanline;

  // 8. Subtle center glow
  float center_glow = smoothstep(0.4, 0.0, dist) * spec * 0.2;
  final_color += color * center_glow;

  fragColor = vec4(final_color, 1.0);
}
