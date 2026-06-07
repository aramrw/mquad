#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;

in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 80;
const float MAX_DIST = 60.0;
const float SURF_DIST = 0.01;

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

// Generates the winding path of the tunnel
vec3 path(float z) {
    float x = sin(z * 0.15) * 3.0 + cos(z * 0.05) * 1.5;
    float y = cos(z * 0.1) * 3.0 + sin(z * 0.08) * 1.5;
    return vec3(x, y, z);
}

float GetDist(vec3 p) {
    vec3 p_path = path(p.z);
    
    // Shift coordinate system to follow the path
    vec2 q = p.xy - p_path.xy;
    
    // Add a slow barrel roll to the tunnel walls
    q *= rot(p.z * 0.05 + Time * 0.2);
    
    float a = atan(q.y, q.x);
    float r = length(q);
    
    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float mids = texture(AudioTexture, vec2(0.5, 0.75)).r;
    
    // The tunnel breathes to the bass
    float tunnelRadius = 3.5 + sin(p.z * 1.5 - Time * 8.0) * 0.8 * bass;
    
    // Add high-frequency audio ripples to the walls
    float ripple = sin(p.z * 8.0 - Time * 12.0) * cos(a * 6.0) * 0.3 * mids;
    
    // We are inside the tunnel, so distance is Radius - our distance from center
    return tunnelRadius - r + ripple;
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float treble = texture(AudioTexture, vec2(0.9, 0.75)).r;

    // Camera moves forward on the Z axis, speed affected by bass
    float speed = Time * 8.0;
    
    // Temporary speed boost when bass hits hard
    if (bass > 0.6) {
        speed += pow(bass - 0.6, 2.0) * 20.0;
    }
    
    vec3 ro = path(speed);
    vec3 lookAt = path(speed + 3.0);
    
    // Standard camera matrix
    vec3 forward = normalize(lookAt - ro);
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), forward));
    vec3 up = cross(forward, right);
    
    // Roll the camera based on treble hits
    float roll = Time * 0.3 + sin(Time) * 0.2 + treble * 0.5;
    vec2 pRot = p * rot(roll);
    
    vec3 rd = normalize(forward + right * pRot.x + up * pRot.y);

    float dO = 0.0;
    float glow = 0.0;
    vec3 pHit;
    
    // Raymarching loop
    for(int i = 0; i < MAX_STEPS; i++) {
        pHit = ro + rd * dO;
        float dS = GetDist(pHit);
        
        // Accumulate volumetric glow as the ray passes near the walls
        glow += 0.015 / (0.01 + abs(dS));
        
        dO += dS;
        if(dO > MAX_DIST || abs(dS) < SURF_DIST) break;
    }

    vec3 color = vec3(0.0);
    
    if(dO < MAX_DIST) {
        float z = pHit.z;
        float a = atan(pHit.y - path(z).y, pHit.x - path(z).x);
        
        // Map texture coordinates to the tunnel walls
        vec2 gridUv = vec2(a * 5.0 / 3.14159, z * 0.5 - Time * 2.0);
        
        // Draw a wireframe / tech grid
        vec2 grid = fract(gridUv);
        float line = smoothstep(0.0, 0.05, grid.x) * smoothstep(1.0, 0.95, grid.x)
                   * smoothstep(0.0, 0.05, grid.y) * smoothstep(1.0, 0.95, grid.y);
        
        // Sample audio mapped to the Z-axis of the tunnel
        float audioSample = texture(AudioTexture, vec2(fract(z * 0.02), 0.75)).r;
        
        // Shifting Cyberpunk color palette
        vec3 colCyan = vec3(0.0, 0.8, 1.0);
        vec3 colPink = vec3(1.0, 0.0, 0.6);
        vec3 baseColor = mix(colCyan, colPink, sin(z * 0.1 - Time * 0.5) * 0.5 + 0.5);
        
        // Only light up the grid lines, driven by audio
        color = baseColor * (1.0 - line) * (0.2 + audioSample * 8.0);
        
        // Darken as it goes into the distance
        float fog = exp(-dO * 0.04);
        color *= fog;
    }
    
    // Add Volumetric Glow to the center of the tunnel
    vec3 glowColor = mix(vec3(0.6, 0.0, 1.0), vec3(0.0, 1.0, 0.8), sin(Time * 0.4) * 0.5 + 0.5);
    // Multiply glow by treble for massive flashes
    color += glowColor * glow * 0.04 * (1.0 + treble * 3.0);

    // Center hyper-drive flash
    color += vec3(1.0) * pow(treble, 3.0) * 0.5 * glow;

    fragColor = vec4(color, 1.0);
}