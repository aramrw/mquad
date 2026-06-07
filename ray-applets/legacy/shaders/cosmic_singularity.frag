#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    float bass = texture(AudioTexture, vec2(0.05, 0.5)).r;
    float mids = texture(AudioTexture, vec2(0.3, 0.5)).r;
    float treble = texture(AudioTexture, vec2(0.7, 0.5)).r;

    vec3 color = vec3(0.0);
    
    // Vortex parameters
    float angle = atan(p.y, p.x);
    float radius = length(p);
    
    // Distort space based on audio
    vec2 pDistorted = p;
    pDistorted *= rot(Time * (0.2 + bass * 0.5) + radius * (2.0 + mids * 3.0));
    
    // Fractal iterations
    float intensity = 0.0;
    vec2 f = pDistorted;
    
    for(float i = 0.0; i < 8.0; i++) {
        float fAudio = texture(AudioTexture, vec2(i / 10.0, 0.5)).r;
        f = abs(f) - (0.1 + fAudio * 0.15);
        f *= rot(Time * 0.1 + i);
        f *= 1.2 + bass * 0.2;
        
        float d = length(f) - (0.01 + treble * 0.05);
        intensity += 0.01 / abs(d);
    }
    
    // Event horizon (black hole center)
    float horizon = smoothstep(0.1 + bass * 0.1, 0.15 + bass * 0.1, radius);
    intensity *= horizon;
    
    // Accretion disk glow
    float disk = smoothstep(0.6, 0.1, radius) * horizon;
    
    // Color mapping
    vec3 baseColor = 0.5 + 0.5 * cos(Time + vec3(0, 2, 4) + angle + radius * 5.0);
    color = baseColor * intensity * (0.5 + mids);
    
    // Add disk glow
    color += vec3(1.0, 0.5, 0.2) * disk * bass * 2.0;
    
    // Add stars pulling in
    float starAngle = angle + Time * 2.0 / (radius + 0.1);
    vec2 starPos = vec2(cos(starAngle), sin(starAngle)) * fract(radius - Time * 0.5);
    float stars = smoothstep(0.98, 1.0, fract(sin(dot(starPos, vec2(12.9898, 78.233))) * 43758.5453));
    color += stars * treble * 2.0 * horizon;

    // Pulse effect
    color *= 1.0 + bass * exp(-radius * 3.0);

    fragColor = vec4(color, 1.0);
}
