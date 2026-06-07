#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 60;
const float MAX_DIST = 20.0;
const float SURF_DIST = 0.01;

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

float GetDist(vec3 p) {
    float bass = texture(AudioTexture, vec2(0.1, 0.5)).r;
    float mids = texture(AudioTexture, vec2(0.5, 0.5)).r;
    float treble = texture(AudioTexture, vec2(0.9, 0.5)).r;

    vec3 q = p;
    // Twist space
    q.xz *= rot(q.y * (0.5 + mids * 2.0) + Time);
    q.xy *= rot(Time * 0.5);

    // Sphere base
    float d1 = length(q) - (1.0 + bass * 0.5);
    
    // Orbiting particles
    vec3 p2 = p;
    p2.xz *= rot(Time * 2.0);
    p2.xy *= rot(Time * 1.5);
    float d2 = length(abs(p2) - (1.5 + mids)) - (0.2 + treble * 0.3);

    // Sine wave displacement
    float disp = sin(p.x * 5.0 + Time * 3.0) * sin(p.y * 5.0 + Time * 3.0) * sin(p.z * 5.0 + Time * 3.0);
    
    float final_d = smin(d1, d2, 0.5) + disp * (0.1 + bass * 0.2);
    
    // Outer shell
    float shell = length(p) - (3.0 + bass);
    shell = abs(shell) - 0.05; // hollow shell
    // add holes to shell
    shell = max(shell, -(length(q.xy) - 1.0));
    shell = max(shell, -(length(q.xz) - 1.0));

    return min(final_d, shell);
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    float bass = texture(AudioTexture, vec2(0.1, 0.5)).r;
    float mids = texture(AudioTexture, vec2(0.5, 0.5)).r;
    float treble = texture(AudioTexture, vec2(0.9, 0.5)).r;

    vec3 ro = vec3(0.0, 0.0, -5.0);
    ro.xz *= rot(Time * 0.2);
    vec3 rd = normalize(vec3(p, 1.0));
    rd.xz *= rot(Time * 0.2);

    float dO = 0.0;
    float glow = 0.0;

    for(int i = 0; i < MAX_STEPS; i++) {
        vec3 pStep = ro + rd * dO;
        float dS = GetDist(pStep);
        
        // Volumetric glow accumulation
        glow += 0.01 / (0.01 + abs(dS));
        
        dO += dS;
        if(dO > MAX_DIST || dS < SURF_DIST) break;
    }

    vec3 color = vec3(0.0);
    
    if(dO < MAX_DIST) {
        // Surface color
        float depth = 1.0 - dO / MAX_DIST;
        color = vec3(0.1, 0.8, 0.5) * depth * (1.0 + bass);
    }

    // Add glowing aura
    vec3 auraColor = mix(vec3(0.1, 0.2, 0.9), vec3(0.9, 0.1, 0.2), sin(Time * 0.5) * 0.5 + 0.5);
    color += auraColor * glow * (0.3 + mids * 0.5);
    
    // Center flash on treble
    color += vec3(1.0, 1.0, 1.0) * glow * glow * 0.005 * treble;

    fragColor = vec4(color, 1.0);
}
