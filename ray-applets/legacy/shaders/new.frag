#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;

in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 100;
const float MAX_DIST = 80.0;
const float SURF_DIST = 0.01;

// Strict Stylized Palette (Drawing / Comic style)
vec3 colPaper = vec3(0.95, 0.92, 0.84);
vec3 colInk = vec3(0.08, 0.08, 0.10);
vec3 colBlue = vec3(0.15, 0.45, 0.65);
vec3 colRed = vec3(0.85, 0.25, 0.20);
vec3 colGold = vec3(0.95, 0.75, 0.10);

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

// 3D Box distance function
float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Calculate the height of a specific box based on audio
float getHeight(vec2 id) {
    float norm_x = clamp(abs(id.x) / 20.0, 0.0, 1.0);
    float spectrum = texture(AudioTexture, vec2(norm_x, 0.75)).r;
    float r = hash(id);
    
    // Base audio response
    float audio_h = spectrum * 8.0;
    
    // The "Pop" Threshold: Amplify the loudest hits for extra depth
    float cutoff = 0.65;
    if (spectrum > cutoff) {
        // Exponentially boost the volume that exceeds the cutoff
        audio_h += pow(spectrum - cutoff, 1.5) * 40.0;
    }
    
    float h = 0.2 + audio_h; 
    h *= (0.4 + r * 0.6);
    
    // Center bass pulse
    if (abs(id.x) <= 2.0) {
        float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
        float bass_boost = bass * 3.0;
        
        // Massive pop for the heavy kicks down the center
        if (bass > 0.7) {
            bass_boost += pow(bass - 0.7, 1.5) * 50.0;
        }
        h += bass_boost;
    }
    return h;
}

// Map the 3D world
float GetDist(vec3 p) {
    float d = p.y; 
    vec2 id = floor(p.xz);
    
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 cur_id = id + vec2(x, y);
            vec2 q = p.xz - (cur_id + 0.5); 
            float h = getHeight(cur_id);
            
            vec3 boxPos = vec3(q.x, p.y - h * 0.5, q.y);
            float dBox = sdBox(boxPos, vec3(0.4, h * 0.5, 0.4));
            d = min(d, dBox);
        }
    }
    return d;
}

// Calculate normal
vec3 GetNormal(vec3 p, vec3 ro) {
    // We need a localized warped distance function for normal calculation
    // because the world is bent!
    float e = 0.01;
    
    vec3 px = p + vec3(e, 0, 0);
    float distZ_px = px.z - ro.z;
    px.y -= distZ_px * distZ_px * 0.005;
    px.y -= (px.x * px.x) * 0.02;
    
    vec3 pnx = p - vec3(e, 0, 0);
    float distZ_pnx = pnx.z - ro.z;
    pnx.y -= distZ_pnx * distZ_pnx * 0.005;
    pnx.y -= (pnx.x * pnx.x) * 0.02;

    vec3 py = p + vec3(0, e, 0);
    float distZ_py = py.z - ro.z;
    py.y -= distZ_py * distZ_py * 0.005;
    py.y -= (py.x * py.x) * 0.02;
    
    vec3 pny = p - vec3(0, e, 0);
    float distZ_pny = pny.z - ro.z;
    pny.y -= distZ_pny * distZ_pny * 0.005;
    pny.y -= (pny.x * pny.x) * 0.02;

    vec3 pz = p + vec3(0, 0, e);
    float distZ_pz = pz.z - ro.z;
    pz.y -= distZ_pz * distZ_pz * 0.005;
    pz.y -= (pz.x * pz.x) * 0.02;
    
    vec3 pnz = p - vec3(0, 0, e);
    float distZ_pnz = pnz.z - ro.z;
    pnz.y -= distZ_pnz * distZ_pnz * 0.005;
    pnz.y -= (pnz.x * pnz.x) * 0.02;

    vec3 n = vec3(
        GetDist(px) - GetDist(pnx),
        GetDist(py) - GetDist(pny),
        GetDist(pz) - GetDist(pnz)
    );
    return normalize(n);
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;
    
    // 1. Fisheye Lens Distortion on the screen coordinates
    float len = length(p);
    // Push the edges of the screen outwards, intensifying at the borders
    p = p * (1.0 + len * len * 0.25);

    // Camera Setup
    vec3 ro = vec3(0.0, 18.0, Time * 12.0); 
    vec3 lookAt = ro + vec3(8.0, -12.0, 12.0); 
    
    vec3 forward = normalize(lookAt - ro);
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), forward));
    vec3 up = cross(forward, right);
    vec3 rd = normalize(forward + right * p.x + up * p.y);

    float dO = 0.0;
    vec3 pHit;
    for(int i = 0; i < MAX_STEPS; i++) {
        pHit = ro + rd * dO;
        
        // 2. World Warping (Inception-style bending)
        // Bend the world upwards (positive Y) based on distance from the camera's Z position
        // and push the edges of the city up based on the X distance
        vec3 warpPos = pHit;
        float distZ = warpPos.z - ro.z;
        warpPos.y -= distZ * distZ * 0.005; // Bend forward horizon up
        warpPos.y -= (warpPos.x * warpPos.x) * 0.02; // Bend side horizons up
        
        float dS = GetDist(warpPos);
        
        // When warping space, we need to slow down the raymarcher slightly
        // so it doesn't overshoot the distorted geometry.
        dO += dS * 0.8; 
        
        if(dO > MAX_DIST || abs(dS) < SURF_DIST) break;
    }

    vec3 color = colPaper; // Default background is paper
    
    if(dO < MAX_DIST) {
        vec3 n = GetNormal(pHit, ro);
        
        // Since we warped the world, we need to map the hit point back to the 
        // warped coordinate system to get the correct cell ID for coloring.
        vec3 warpPos = pHit;
        float distZ = warpPos.z - ro.z;
        warpPos.y -= distZ * distZ * 0.005;
        warpPos.y -= (warpPos.x * warpPos.x) * 0.02;
        
        vec2 id = floor(warpPos.xz);
        float r = hash(id);
        
        // 1. Strict Color Blocking (Flat coloring)
        vec3 albedo = colPaper;
        if (r < 0.15) albedo = colInk;
        else if (r < 0.40) albedo = colRed;
        else if (r < 0.65) albedo = colBlue;
        
        // Audio spikes get highlighted in Gold
        float h = getHeight(id);
        if (h > 2.0 && r > 0.4) {
            albedo = colGold;
        }

        // 2. Binary Shading (Cel-shaded shadow)
        vec3 lightDir = normalize(vec3(0.8, 0.5, -0.4));
        float diff = dot(n, lightDir);
        float isLit = step(0.0, diff); // 1.0 if facing light, 0.0 if not
        
        if (albedo == colInk) {
            color = colInk;
        } else {
            // Shadow is a flat 30% brightness version of the color
            vec3 shadowColor = albedo * 0.3;
            color = mix(shadowColor, albedo, isLit);
        }
        
        // Force all top-facing roofs to be fully lit to look like an architectural drawing
        if (n.y > 0.5) {
            color = albedo;
        }

        // 3. Strict Ink Outlines
        float outline = 0.0;
        vec2 lPos = abs(fract(warpPos.xz) - 0.5); // Local UV inside the grid cell
        
        // Detect edges based on which face of the box we are looking at
        if (n.y > 0.5) {
            // Roof borders
            outline = step(0.36, max(lPos.x, lPos.y));
        } else if (abs(n.x) > 0.5) {
            // Side wall borders (Z axis)
            outline = step(0.36, lPos.y); 
        } else {
            // Side wall borders (X axis)
            outline = step(0.36, lPos.x);
        }
        
        // Horizontal lines for the top and bottom of the side walls
        if (n.y < 0.5) {
            float topEdge = step(abs(warpPos.y - h), 0.05);
            float bottomEdge = step(warpPos.y, 0.05);
            outline = max(outline, max(topEdge, bottomEdge));
        }
        
        // Apply solid ink outline
        if (outline > 0.0) {
            color = colInk;
        }
    }

    // 4. Stylized Cutoff Fog (No smooth fading)
    float fog = exp(-dO * 0.035);
    // Sharp binary cutoff: If it's too far, it instantly pops to the paper color
    fog = step(0.15, fog); 
    
    color = mix(colPaper, color, fog);

    fragColor = vec4(color, 1.0);
}