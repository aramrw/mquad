#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;
uniform float AudioLevel;
in vec2 uv;
out vec4 fragColor;

// --- CYBER CITY ADAPTATION (AUDIO REACTIVE) ---
// Based on Hazel Quantock's Cyber City
// Mapping building heights and glitch effects to real-time audio bands

#define M1 1597334677U
#define M2 3812015801U
#define M3 3299493293U
#define F0 exp2(-32.)

#define hash(n) n*(n^(n>>15))
#define coord2(p) (p.x*M1^p.y*M2)

float hash1(uint n){return float(hash(n))*F0;}

// Map city height to Bass and Mids
float CellHeight(ivec2 cell, float bass, float mids) {
    float h = hash1(coord2(uvec2(cell + 0x80000000)));
    // Buildings "bounce" with the beat
    float reactive_h = pow(h + 0.0000001, 5.0) * (8.0 + bass * 15.0 + mids * 5.0);
    return reactive_h;
}

struct Ray { vec3 start, dir; };
struct Intersection { float t; ivec2 cell; vec3 normal; };

Intersection traceGridCity(Ray ray, float bass, float mids) {
    vec3 pos = ray.start;
    ivec2 cell = ivec2(floor(pos.xz));
    ivec2 cellDir = ivec2(sign(ray.dir.xz));
    float t = 0.01;
    vec3 face = vec3(0);

    for (int i = 0; i < 60; i++) {
        float h = CellHeight(cell, bass, mids);
        if (pos.y < h) break;

        vec2 boundary = vec2(cell + max(cellDir, 0));
        float deltax = (boundary.x - pos.x) / ray.dir.x;
        float deltaz = (boundary.y - pos.z) / ray.dir.z;
        float delta = min(deltax, deltaz);

        if (deltax < deltaz) {
            face = vec3(sign(ray.dir.x), 0, 0);
            cell.x += cellDir.x;
        } else {
            face = vec3(0, 0, sign(ray.dir.z));
            cell.y += cellDir.y;
        }

        t += delta;
        pos = ray.start + t * ray.dir;
        
        if (t > 50.0) return Intersection(1e30, ivec2(0), vec3(0));
    }
    return Intersection(t, cell, face);
}

void main() {
    // 1. Sample Audio Bands
    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float mids = texture(AudioTexture, vec2(0.4, 0.75)).r;
    float treble = texture(AudioTexture, vec2(0.8, 0.75)).r;

    // 2. Camera Setup (Driven by Time and Audio)
    float tilt = Time * 0.2 + bass * 0.1;
    vec3 camStart = vec3(0.5, 8.0 + bass * 4.0, Time * 4.0);
    
    // Treble-driven camera jitter
    vec2 jitter = vec2(sin(Time * 50.0), cos(Time * 50.0)) * treble * 0.05;
    vec2 screen_p = (uv * 2.0 - 1.0) + jitter;
    screen_p.x *= Resolution.x / Resolution.y;

    vec3 rd = normalize(vec3(screen_p, 1.2));
    
    // Barrel distortion (Grunge)
    rd.xy *= mix(1.0, 0.5 + pow(length(rd.xy) / 0.8, 5.0), 0.2);
    rd = normalize(rd);

    // Camera rotation
    float s = sin(tilt), c = cos(tilt);
    mat2 rot = mat2(c, -s, s, c);
    rd.yz *= rot;
    rd.xz *= rot;

    // 3. Trace City
    Intersection it = traceGridCity(Ray(camStart, rd), bass, mids);

    vec3 atmosColour = vec3(0.01, 0.02, 0.05) + vec3(bass * 0.05, 0.0, mids * 0.1);
    vec3 color = atmosColour;

    if (it.t < 50.0) {
        vec3 pos = camStart + rd * it.t;
        
        // Window lights: flickering based on treble/mids
        float windows = step(0.5, fract(pos.y * 5.0)) * step(0.8, hash1(coord2(uvec2(it.cell))));
        vec3 lightCol = 0.5 + 0.5 * cos(Time + float(it.cell.x) + vec3(0, 2, 4));
        
        // Windows pulse with music
        color = lightCol * windows * (10.0 * mids + 5.0 * treble);
        
        if (it.normal.y > 0.0) color = vec3(0); // Top of buildings dark
        
        // Fog/Atmosphere distance fade
        color = mix(atmosColour, color, exp2(-it.t / 12.0));
    }

    // 4. Post-processing (Grunge & Glitch)
    
    // Chromatic aberration driven by treble
    float ca = treble * 0.02;
    float r = color.r;
    float g = color.g;
    float b = color.b; // In a single pass we can't easily sample offset ray results, 
                       // but we can jitter the final color bands
    
    // Scanlines
    color *= 0.9 + 0.1 * sin(uv.y * 300.0 + Time * 20.0);
    
    // Flash on bass hits
    color += vec3(0.05, 0.02, 0.1) * bass;

    // Exposure adjustment
    color *= mix(1.0, 2.0, mids);

    fragColor = vec4(pow(color, vec3(1.0 / 2.2)), 1.0);
}
