#version 100
precision lowp float;
varying vec2 uv;
uniform float Time;
uniform sampler2D AudioTexture;
void main() {
    float audio = texture2D(AudioTexture, vec2(uv.x, 0.5)).r;
    float line = abs(uv.y - audio);
    float glow = 0.01 / line;
    gl_FragColor = vec4(vec3(glow) * vec3(0.0, 1.0, 0.5), 1.0);
}
