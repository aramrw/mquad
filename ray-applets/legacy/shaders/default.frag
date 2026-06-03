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
    float waveform = texture(AudioTexture, vec2(norm_angle, 0.25)).r;
    float spectrum = texture(AudioTexture, vec2(norm_angle, 0.75)).r;

    // 4. Create the 'Spiky' deformation
    // Base radius + waveform pulse + high frequency spikes
    float base_radius = 0.3;
    float spikiness = spectrum * 0.4;
    float target_dist = base_radius + waveform * 0.1 + spikiness;

    // 5. Draw the ring with glow
    float ring_thickness = 0.02;
    float ring_edge = abs(dist - target_dist);
    float glow = 0.02 / (ring_edge + 0.005);
    
    // 6. Color palette
    vec3 color = 0.5 + 0.5 * cos(Time + angle + vec3(0, 2, 4));
    
    // Add the glow
    vec3 final_color = color * glow;

    // 7. Background: subtle spectral cloud
    float bg_pulse = texture(AudioTexture, vec2(0.1, 0.75)).r;
    final_color += color * 0.1 * bg_pulse * (1.0 - dist);

    fragColor = vec4(final_color, 1.0);
}
