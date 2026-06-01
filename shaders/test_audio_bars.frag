#version 150
precision mediump float;

uniform vec2 Resolution;
uniform sampler2D AudioTexture;
in vec2 uv;
out vec4 fragColor;

void main() {
    // 1. Get the frequency sample (Spectrum row is at Y=0.75)
    float audio = texture(AudioTexture, vec2(uv.x, 0.75)).r;

    // 2. Draw a vertical bar
    vec3 color = vec3(0.0);
    
    if (uv.y < audio) {
        color = vec3(audio, 1.0 - audio, 0.5);
    }

    // 3. Add a thin white line at the very top of the bar for clarity
    float line_dist = abs(uv.y - audio);
    if (line_dist < 0.005) {
        color = vec3(1.0);
    }

    fragColor = vec4(color, 1.0);
}
