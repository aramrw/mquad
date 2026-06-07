#version 150
precision mediump float;

uniform sampler2D AudioTexture;
uniform vec2 Resolution;
uniform float Time;
in vec2 uv;
out vec4 fragColor;

float hash(float x) {
  return fract(sin(x * 12.9898) * 43758.5453);
}

// A procedural palette function to generate distinct colors based on the layer index.
// This avoids using slow, messy if/else statements inside the rendering loop.
vec3 getLayerColor(float index) {
  // Shifts the RGB values cleanly as the index goes from 0 to 4
  return 0.5 + 0.5 * cos(vec3(0.0, 1.0, 2.0) + index * 1.2);
}

void main() {
  vec3 color = vec3(0.0);

  // 5 distinct layers: 0=Bass, 1=Low Mids, 2=Mids, 3=High Mids, 4=Highs
  for (float i = 0.0; i < 5.0; i++) {

    // 1. FREQUENCY MAPPING:
    // i=0 maps to X=0.05 (Bass). i=4 maps to X=0.85 (Highs).
    float fftX = 0.05 + (i * 0.2);
    float audioLevel = texture(AudioTexture, vec2(fftX, 0.75)).r;

    // 2. SCALE MAPPING:
    // Higher frequencies get denser, thinner rain columns.
    float scale = 15.0 + (i * 15.0);

    // 3. SPEED MAPPING:
    // i=0 (Bass) falls slowly. i=4 (Highs) falls incredibly fast.
    float speed = 0.4 + (i * 1.5) + (audioLevel * 0.2);

    vec2 st = uv * vec2(scale, 1.0);
    st.y += Time * speed;

    float columnID = floor(st.x);
    st.y += hash(columnID + i * 200.0) * 40.0;

    vec2 grid = fract(st);
    float localX = 0.5 + (hash(columnID * 3.14) - 0.5) * 0.6;

    // Make higher frequency streaks physically thinner
    float thickness = 0.05 - (i * 0.008);
    float xDist = abs(grid.x - localX);
    float drop = smoothstep(thickness, 0.0, xDist);

    drop *= smoothstep(0.8, 0.1, grid.y);
    drop *= smoothstep(0.0, 0.1, grid.y);

    float activeDrop = step(0.6, hash(columnID * 13.0 + floor(st.y)));

    // 4. COLOR MAPPING:
    // Intensity is driven strictly by this specific layer's frequency band
    float intensity = 0.1 + (audioLevel * 3.0);
    vec3 layerColor = getLayerColor(i) * intensity;

    // Blend the active drop into the main color output
    color += drop * activeDrop * layerColor;
  }

  // 5. GLOBAL BASS FLASH:
  // A subtle environmental reaction when the sub-bass hits hard
  float globalBass = texture(AudioTexture, vec2(0.05, 0.75)).r;
  if (globalBass > 0.8) {
    color += vec3(0.05, 0.0, 0.05) * (globalBass - 0.8);
  }

  fragColor = vec4(color, 1.0);
}
