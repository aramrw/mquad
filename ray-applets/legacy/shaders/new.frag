#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;

in vec2 uv;
out vec4 fragColor;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

void main() {
    vec2 p = uv.xy;
    p.x *= Resolution.x / Resolution.y;

    // Fixed grid size. Time % 128.0 creates a violent snap when it resets.
    // A static or smoothly oscillating grid size provides a superior foundation.
    float gridSize = 64.0; 

    vec2 gridUv = fract(p * gridSize);
    vec2 cellId = floor(p * gridSize);

    // 1. Audio Sampling Strategy
    // Normalize the X coordinate of the cell to [0.0, 1.0] for the texture lookup
    float norm_x = cellId.x / (Resolution.x / Resolution.y * gridSize);
    norm_x = clamp(norm_x, 0.0, 1.0);

    // Sample the spectrum (high frequencies) mapped to columns
    float spectrum = texture(AudioTexture, vec2(norm_x, 0.75)).r;
    
    // Sample the waveform (low frequencies) globally for structural pulsing
    float pulse = texture(AudioTexture, vec2(0.1, 0.25)).r;

    float randVal = hash(cellId);

    // 2. Audio-Driven Logic
    // Shift the pattern thresholds based on the spectrum intensity in this column
    float dynamicRand = fract(randVal + (spectrum * 0.6));

    vec3 colorParchment = vec3(0.93, 0.89, 0.81);
    vec3 colorBrown = vec3(0.35, 0.20, 0.10);
    vec3 colorBlack = vec3(0.05, 0.05, 0.05);

    vec3 finalColor = colorParchment;

    if (dynamicRand < 0.20) {
        finalColor = colorBlack;
        
    } else if (dynamicRand < 0.50) {
        vec2 subUv = fract(gridUv * 4.0);
        // The global audio pulse alters the thickness of the grid lines
        float edgeThickness = 0.05 + (pulse * 0.15);
        float edge = step(edgeThickness, subUv.x) * step(edgeThickness, subUv.y);
        finalColor = mix(colorBrown, colorParchment, edge);
        
    } else if (dynamicRand < 0.80) {
        vec2 subUv = fract(gridUv * 8.0);
        float edge = step(0.1, subUv.x) * step(0.1, subUv.y);
        finalColor = mix(colorBlack, colorParchment, edge);
        
    } else {
        if (gridUv.x > gridUv.y) {
            finalColor = colorBrown;
        } else {
            finalColor = colorParchment;
        }
    }

    fragColor = vec4(finalColor, 1.0);
}
