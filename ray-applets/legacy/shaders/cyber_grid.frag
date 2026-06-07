#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 100;
const float MAX_DIST = 50.0;
const float SURF_DIST = 0.01;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453123);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

float GetDist(vec3 p) {
    float bass = texture(AudioTexture, vec2(0.1, 0.5)).r;
    // Displaced terrain
    float h = noise(p.xz * 0.5 - vec2(0.0, Time * 2.0)) * (0.5 + bass * 2.5);
    h += noise(p.xz * 2.0 - vec2(0.0, Time * 4.0)) * 0.2; // details
    return p.y + 1.5 - h;
}

float RayMarch(vec3 ro, vec3 rd) {
    float dO = 0.0;
    for(int i = 0; i < MAX_STEPS; i++) {
        vec3 p = ro + rd * dO;
        float dS = GetDist(p);
        dO += dS;
        if(dO > MAX_DIST || dS < SURF_DIST) break;
    }
    return dO;
}

vec3 GetNormal(vec3 p) {
    float d = GetDist(p);
    vec2 e = vec2(0.01, 0);
    vec3 n = d - vec3(
        GetDist(p - e.xyy),
        GetDist(p - e.yxy),
        GetDist(p - e.yyx)
    );
    return normalize(n);
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    float bass = texture(AudioTexture, vec2(0.1, 0.5)).r;
    float mids = texture(AudioTexture, vec2(0.4, 0.5)).r;
    float treble = texture(AudioTexture, vec2(0.8, 0.5)).r;

    // Camera shake based on treble
    vec3 ro = vec3(0.0, 0.5 + bass * 0.5, 0.0);
    ro.x += sin(Time * 50.0) * treble * 0.05;
    ro.y += cos(Time * 60.0) * treble * 0.05;
    
    vec3 rd = normalize(vec3(p.x, p.y - 0.2, 1.0));

    float d = RayMarch(ro, rd);
    vec3 color = vec3(0.0);

    if(d < MAX_DIST) {
        vec3 pHit = ro + rd * d;
        vec3 n = GetNormal(pHit);
        
        // Grid pattern mapping
        vec2 grid = fract(pHit.xz * 2.0 - vec2(0.0, Time * 2.0));
        float line = smoothstep(0.0, 0.05, grid.x) * smoothstep(1.0, 0.95, grid.x)
                   * smoothstep(0.0, 0.05, grid.y) * smoothstep(1.0, 0.95, grid.y);
        line = 1.0 - line; // invert so lines are 1

        float heightNorm = clamp((pHit.y + 1.5) / 3.0, 0.0, 1.0);
        
        vec3 lineColor = mix(vec3(0.0, 1.0, 0.8), vec3(1.0, 0.0, 0.8), heightNorm + mids);
        vec3 baseColor = vec3(0.05, 0.0, 0.1);
        
        color = mix(baseColor, lineColor * (1.0 + bass * 2.0), line);
        
        // Distance fog
        float fog = exp(-d * 0.08);
        color *= fog;
    } else {
        // Sky with sun
        float sunMask = length(p - vec2(0.0, 0.2));
        float sun = smoothstep(0.4, 0.38, sunMask);
        float sunGlow = smoothstep(1.5, 0.0, sunMask);
        
        // Sun bars
        float bars = step(0.5, sin(p.y * 40.0 - Time * 2.0));
        sun *= bars;
        
        vec3 sunColor = mix(vec3(1.0, 0.8, 0.0), vec3(1.0, 0.0, 0.5), p.y + 0.5);
        
        color = sun * sunColor * (1.0 + bass);
        color += sunGlow * vec3(0.8, 0.2, 0.5) * mids * 0.5;
        
        // Background fog matching terrain
        float horizon = exp(-abs(p.y - 0.2) * 4.0);
        color += vec3(1.0, 0.0, 0.8) * horizon * bass;
    }

    // Vignette and scanlines
    float scanline = sin(uv.y * Resolution.y * 2.0) * 0.04;
    color -= scanline;
    color *= 1.0 - dot(p, p) * 0.15;

    fragColor = vec4(color, 1.0);
}
