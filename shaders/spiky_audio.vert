#version 150
in vec3 position;
in vec2 texcoord;
out vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
uniform sampler2D AudioTexture;
uniform float AudioLevel;

void main() {
    vec3 pos = position;
    
    // Sample frequency spectrum at Y=0.75 for vertex spikes
    float spike = texture(AudioTexture, vec2(texcoord.x, 0.75)).r;
    
    // Deform the mesh: normalize(position) gives direction from center for sphere/cube
    float deformation = (AudioLevel * 0.5) + (spike * 1.5);
    pos += normalize(position) * deformation;

    gl_Position = Projection * Model * vec4(pos, 1.0);
    uv = texcoord;
}
