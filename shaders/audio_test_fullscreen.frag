#version 150
precision mediump float;

uniform sampler2D AudioTexture;
uniform vec2 Resolution;
uniform float Time;
in vec2 uv;
out vec4 fragColor;

void main() {
  // 1. Get the audio sample for this horizontal position (Waveform row is at Y=0.25)
  float audio = texture(AudioTexture, vec2(uv.x, 0.25)).r;

  // 2. Background: Subtle pulse based on bass (Spectrum row is at Y=0.75)
  float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
  vec3 color = vec3(bass * 0.2, 0.0, bass * 0.3);

  // 3. Draw the actual waveform
  // We create a line where Y = AudioValue
  float thickness = 0.02;
  float dist = abs(uv.y - audio);
  float line = smoothstep(thickness, 0.0, dist);

  // Make the line glow cyan
  color += line * vec3(0.0, 1.0, 1.0);

  // 4. Fill everything below the line with a darker green
  if (uv.y < audio) {
    color += vec3(0.0, audio * 0.3, audio * 0.2);
  }

  // 5. Add a "beat flash" - if bass is high, flash the whole screen slightly
  if (bass > 0.8) {
    color += vec3(0.1);
  }

  fragColor = vec4(color, 1.0);
}
