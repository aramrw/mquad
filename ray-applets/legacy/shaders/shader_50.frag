#version 150
precision lowp float;
in vec2 uv;
out vec4 fragColor;
uniform sampler2D AudioTexture;
void main() {
    float audio = texture(AudioTexture, vec2(uv.x, 0.5)).r;
    fragColor = vec4(sin(audio), 0.2, clamp(audio, 0.0, 1.0), 1.0);
}
