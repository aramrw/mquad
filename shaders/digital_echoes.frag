#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

// Unique "Digital Echoes" Shader
// Evolution of the user's oscillating line into a frequency-aware data-stream

void main() {
    // 1. Center coordinates
    vec2 p = uv * 2.0 - 1.0;
    p.x *= Resolution.x / Resolution.y;

    // 2. Sample Audio Bands (Spectrum row at Y=0.75)
    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float mids = texture(AudioTexture, vec2(0.4, 0.75)).r;
    float treble = texture(AudioTexture, vec2(0.8, 0.75)).r;
    float wave = texture(AudioTexture, vec2(uv.x, 0.25)).r;

    // 3. Dynamic speed and distortion
    float speed = Time * (0.5 + bass * 0.5);
    
    // Horizontal distortion based on treble jitter
    float jitter = sin(uv.y * 50.0 + Time * 20.0) * treble * 0.05;
    float x_distort = p.x + jitter;

    // 4. Create multiple oscillating "Echo" lines
    vec3 finalColor = vec3(0.0);
    
    for(int i = 0; i < 5; i++) {
        float index = float(i);
        // Each line has a different frequency and phase offset
        float offset = sin(speed + index * 1.2 + x_distort * (2.0 + index)) * (0.3 + mids * 0.2);
        
        // Line thickness reacts to bass
        float thickness = 0.01 + (bass * 0.05) / (index + 1.0);
        
        // The vertical position of the line
        float dist = abs(p.y + offset);
        
        // Sharp edge with a tiny bit of glow
        float intensity = smoothstep(thickness, thickness * 0.5, dist);
        
        // Chromatic Abberation: Offset colors per echo
        vec3 col;
        if(i == 0) col = vec3(0.0, 1.0, 1.0); // Cyan
        else if(i == 1) col = vec3(1.0, 0.0, 0.5); // Magenta
        else if(i == 2) col = vec3(1.0, 0.8, 0.0); // Gold
        else col = vec3(0.5, 0.2, 1.0); // Purple
        
        // Fade echoes as they get further from index 0
        finalColor += col * intensity * (1.0 - index * 0.15);
    }

    // 5. Add "Data Noise" spikes
    if (abs(p.y - (wave - 0.5) * 2.0) < 0.005) {
        finalColor += vec3(1.0) * treble;
    }

    // 6. Background Scanlines / Grid
    float grid = sin(p.x * 20.0) * sin(p.y * 20.0);
    finalColor += vec3(0.02, 0.05, 0.1) * smoothstep(0.98, 1.0, grid) * bass;

    // 7. Subtle Vignette
    finalColor *= 1.2 - length(p) * 0.6;

    fragColor = vec4(finalColor, 1.0);
}
