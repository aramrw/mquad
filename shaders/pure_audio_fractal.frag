#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 80;
const float MAX_DIST = 100.0;
const float SURF_DIST = 0.001;

// Pure Audio Rotation - only moves when sound is present
mat2 Rotate(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

float GetDist(vec3 p) {
    // Sample different parts of the audio spectrum
    float bass = texture(AudioTexture, vec2(0.05, 0.5)).r;
    float mids = texture(AudioTexture, vec2(0.3, 0.5)).r;
    float treble = texture(AudioTexture, vec2(0.8, 0.5)).r;
    
    // Rotation is driven ONLY by mids/treble
    p.xy *= Rotate(mids * 4.0);
    p.yz *= Rotate(treble * 2.0);
    
    // Scale is driven ONLY by bass
    float size = 0.5 + bass * 1.5;
    
    // Fractal-like repetition (KIFS) driven by mids
    for(int i=0; i<3; i++) {
        p = abs(p) - (0.5 + mids * 0.2);
        p.xy *= Rotate(treble * 1.0);
        p.xz *= Rotate(bass * 0.5);
    }
    
    // The final shape: A box that deforms with bass
    vec3 q = abs(p) - size;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float RayMarch(vec3 ro, vec3 rd) {
    float dO = 0.0;
    for(int i=0; i<MAX_STEPS; i++) {
        vec3 p = ro + rd * dO;
        float dS = GetDist(p);
        dO += dS;
        if(dO > MAX_DIST || dS < SURF_DIST) break;
    }
    return dO;
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    // Use treble for camera jitter
    float treble = texture(AudioTexture, vec2(0.9, 0.75)).r;
    vec3 ro = vec3(0.0, 0.0, -5.0 + treble * 0.5);
    vec3 rd = normalize(vec3(p, 1.2));

    float d = RayMarch(ro, rd);
    
    vec3 color = vec3(0.0);
    
    if(d < MAX_DIST) {
        float bass = texture(AudioTexture, vec2(0.1, 0.75)).r;
        float mids = texture(AudioTexture, vec2(0.5, 0.75)).r;
        
        // Lighting based on depth
        float depth = 1.0 - (d / 10.0);
        
        // Color palette driven by audio bands
        color = vec3(bass, mids * 0.5, 0.8) * depth;
        
        // Spectral flash
        color += vec3(0.2, 0.8, 1.0) * pow(treble, 3.0);
    }
    
    // Background glow
    float bass_bg = texture(AudioTexture, vec2(0.01, 0.75)).r;
    color += vec3(0.1, 0.05, 0.2) * bass_bg * (1.0 - length(p));

    fragColor = vec4(color, 1.0);
}
