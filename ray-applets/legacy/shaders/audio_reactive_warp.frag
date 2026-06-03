#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;

in vec2 uv;
out vec4 fragColor;

void main() {
    // Normalizing coordinates to aspect ratio
    vec2 pos = (gl_FragCoord.xy - 0.5 * Resolution.xy) / Resolution.y;
    vec2 uuv = pos;

    // React to sound: Sample the AudioTexture
    float audio = texture(AudioTexture, vec2(abs(uv.x), 0.75)).r;

    // Timing calculations
    float clock = Time / 5.0 + audio * 2.0;
    float ttime = floor(Time) + pow(fract(Time), sin(Time * 1.333) * 0.5);

    // Domain Warping
    pos.x += tan(abs(pos.x) * 5.0 + audio * 3.0);

    // Space Repetition
    float d = fract(10.0 * pos.x + clock);
    d = smoothstep(0.2, 0.10, d);

    // Color Generation
    float p = tan(pos.x + Time + audio * 5.0);
    vec3 bg_color = vec3(0.1, 0.1, 0.1);
    vec3 fg_color = vec3(
        p * 0.1 + step(0.1, abs(cos(clock * 2.0)) * pos.x), 
        p * 0.9 + audio * 0.5, 
        0.1 + step(1.0, sin(clock) * sin(clock) * abs(pos.x))
    );

    vec3 final_color = mix(bg_color, fg_color, d);

    // Alpha modulation
    float alpha = d * sin(ttime + 1.0 - floor(length(floor(uuv * 10.0)))) + (audio * 0.5);

    fragColor = vec4(final_color, alpha);
}
