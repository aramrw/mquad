#version 150
precision mediump float;

uniform sampler2D AudioTexture;
uniform vec2 Resolution;
uniform float Time;
uniform float AudioLevel;
in vec2 uv;
out vec4 fragColor;

void main() {
    // 1. Waveform row at Y=0.25
    float audio = texture(AudioTexture, vec2(uv.x, 0.25)).r;

    // 2. Base background color using spectrum mids at Y=0.75
    float mids = texture(AudioTexture, vec2(0.4, 0.75)).r;
    vec3 color = vec3(0.05, 0.02, 0.1) + vec3(mids * 0.2, 0.0, mids * 0.5);

    // 3. Draw a glowing waveform line
    float thickness = 0.01 + (AudioLevel * 0.05);
    float dist = abs(uv.y - audio);
    float line = smoothstep(thickness, 0.0, dist);
    
    // Line color reacts to volume: cyan -> white
    vec3 lineColor = mix(vec3(0.0, 1.0, 0.8), vec3(1.0, 1.0, 1.0), AudioLevel);
    color += line * lineColor * 2.0;

    // 4. Reactive bloom/glow based on AudioLevel
    float bloom = smoothstep(0.5, 0.0, length(uv - 0.5));
    color += bloom * vec3(0.3, 0.1, 0.6) * AudioLevel;

    // 5. Digital jitter effect on peaks
    if (AudioLevel > 0.7) {
        color.rg += 0.1 * sin(uv.y * 200.0);
    }

    fragColor = vec4(color, 1.0);
}
