#version 100
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
varying vec2 uv;

// Raymarching constants
const int MAX_STEPS = 64;
const float MAX_DIST = 100.0;
const float SURF_DIST = 0.01;

// Audio-reactive transformation
mat2 Rotate(float a) {
  float s = sin(a), c = cos(a);
  return mat2(c, -s, s, c);
}

float GetDist(vec3 p) {
  float audio = texture2D(AudioTexture, vec2(0.1, 0.5)).r;

  // Distort space with audio
  p.xy *= Rotate(audio * 2.0);
  p.xz *= Rotate(audio * 0.3);

  // Create a music-reactive fractal/box thing
  vec3 q = abs(p) - (1.0 + audio * 0.5);
  float box = length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);

  // Add a pulsing sphere inside
  float sphere = length(p) - (0.8 + audio * 0.8);

  return mix(box, sphere, 0.5 + 0.5 * sin(Time));
}

float RayMarch(vec3 ro, vec3 rd) {
  float dO = 0.0;
  for (int i = 0; i < MAX_STEPS; i++) {
    vec3 p = ro + rd * dO;
    float dS = GetDist(p);
    dO += dS;
    if (dO > MAX_DIST || dS < SURF_DIST) break;
  }
  return dO;
}

vec3 GetNormal(vec3 p) {
  float d = GetDist(p);
  vec2 e = vec2(0.01, 0.0);
  vec3 n = d - vec3(
        GetDist(p - e.xyy),
        GetDist(p - e.yxy),
        GetDist(p - e.yyx));
  return normalize(n);
}

void main() {
  vec2 p = (uv.xy * 2.0 - 1.0);
  p.x *= Resolution.x / Resolution.y;

  float audio = texture2D(AudioTexture, vec2(uv.x * 0.5, 0.5)).r;

  // Camera
  vec3 ro = vec3(0.0, 0.0, -4.0);
  vec3 rd = normalize(vec3(p, 1.0));

  float d = RayMarch(ro, rd);

  vec3 color = vec3(0.0);

  if (d < MAX_DIST) {
    vec3 pos = ro + rd * d;
    vec3 n = GetNormal(pos);

    // Lighting
    vec3 lightPos = vec3(1.0, 5.0, -2.0);
    vec3 l = normalize(lightPos - pos);
    float diff = clamp(dot(n, l), 0.0, 1.0);

    // Color shifts with audio
    color = vec3(diff);
    color *= vec3(0.5 + 0.5 * n.x, 0.5 + 0.5 * n.y, 1.0);
    color += vec3(audio * 0.5, 0.0, audio);
  }

  // Atmospheric fog
  color = mix(color, vec3(0.05, 0.05, 0.1), 1.0 - exp(-0.05 * d));

  // Add audio glow
  color += vec3(0.0, 0.2, 0.3) * audio;

  gl_FragColor = vec4(color, 1.0);
}
