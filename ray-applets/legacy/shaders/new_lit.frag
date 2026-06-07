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

// Custom Palette: ["#0d1b2a","#1b263b","#415a77","#778da9","#e0e1dd"]
vec3 col1 = vec3(0.051, 0.106, 0.165); // Darkest Navy
vec3 col2 = vec3(0.106, 0.149, 0.231); // Dark Blue
vec3 col3 = vec3(0.255, 0.353, 0.467); // Slate
vec3 col4 = vec3(0.467, 0.553, 0.663); // Light Slate
vec3 col5 = vec3(0.878, 0.882, 0.867); // Off-White

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Calculate the height and the emissive power of a specific box
float getHeight(vec2 id, out float emission) {
    float norm_x = clamp(abs(id.x) / 20.0, 0.0, 1.0);
    float spectrum = texture(AudioTexture, vec2(norm_x, 0.75)).r;
    float r = hash(id);
    
    float audio_h = spectrum * 8.0;
    float cutoff = 0.60; // Lowered cutoff so things pop easier
    emission = 0.0;
    
    // EXTREME POP MATH
    if (spectrum > cutoff) {
        // Multiplier jumped from 40.0 -> 120.0 for massive pops
        audio_h += pow(spectrum - cutoff, 1.5) * 120.0; 
        emission = pow(spectrum - cutoff, 1.5) * 12.0; 
    }
    
    float h = 0.2 + audio_h; 
    h *= (0.4 + r * 0.6);
    
    if (abs(id.x) <= 2.0) {
        float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
        float bass_boost = bass * 3.0;
        if (bass > 0.65) { // Lowered bass cutoff
            // Multiplier jumped from 50.0 -> 180.0
            bass_boost += pow(bass - 0.65, 1.5) * 180.0;
            emission += pow(bass - 0.65, 1.5) * 20.0;
        }
        h += bass_boost;
    }
    return h;
}

float GetDist(vec3 p) {
    float d = p.y; 
    vec2 id = floor(p.xz);
    
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 cur_id = id + vec2(x, y);
            vec2 q = p.xz - (cur_id + 0.5); 
            float dummy_em;
            float h = getHeight(cur_id, dummy_em);
            
            vec3 boxPos = vec3(q.x, p.y - h * 0.5, q.y);
            float dBox = sdBox(boxPos, vec3(0.4, h * 0.5, 0.4));
            d = min(d, dBox);
        }
    }
    return d;
}

vec3 GetNormal(vec3 p, vec3 ro) {
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

// Ambient occlusion approximation
float calcAO(vec3 pos, vec3 nor) {
    float occ = 0.0;
    float sca = 1.0;
    for(int i=0; i<5; i++) {
        float hr = 0.01 + 0.12*float(i)/4.0;
        vec3 aopos =  nor * hr + pos;
        float dd = GetDist(aopos);
        occ += -(dd-hr)*sca;
        sca *= 0.95;
    }
    return clamp( 1.0 - 3.0*occ, 0.0, 1.0 );    
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;
    
    float len = length(p);
    p = p * (1.0 + len * len * 0.25);

    vec3 ro = vec3(0.0, 18.0, Time * 12.0); 
    vec3 lookAt = ro + vec3(8.0, -12.0, 12.0); 
    
    vec3 forward = normalize(lookAt - ro);
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), forward));
    vec3 up = cross(forward, right);
    vec3 rd = normalize(forward + right * p.x + up * p.y);

    // BARREL ROLL SPIN: 
    // Rotate the ray direction along its local Z axis over time to create a spiraling effect
    float spinSpeed = Time * 0.3 + sin(Time * 0.5) * 0.2; // Slow, slightly swaying roll
    rd.xy *= rot(spinSpeed);

    float dO = 0.0;
    vec3 pHit;
    float glowAcc = 0.0; // Volumetric glow accumulator

    for(int i = 0; i < MAX_STEPS; i++) {
        pHit = ro + rd * dO;
        
        vec3 warpPos = pHit;
        float distZ = warpPos.z - ro.z;
        warpPos.y -= distZ * distZ * 0.005;
        warpPos.y -= (warpPos.x * warpPos.x) * 0.02;
        
        float dS = GetDist(warpPos);
        
        // Accumulate volumetric atmosphere glow as ray passes close to surfaces
        if (dS < 1.0) {
            glowAcc += 0.008 * (1.0 - dS);
        }
        
        dO += dS * 0.8; 
        
        if(dO > MAX_DIST || abs(dS) < SURF_DIST) break;
    }

    vec3 color = vec3(0.0);
    
    if(dO < MAX_DIST) {
        vec3 n = GetNormal(pHit, ro);
        
        vec3 warpPos = pHit;
        float distZ = warpPos.z - ro.z;
        warpPos.y -= distZ * distZ * 0.005;
        warpPos.y -= (warpPos.x * warpPos.x) * 0.02;
        
        vec2 id = floor(warpPos.xz);
        float r = hash(id);
        
        float emission = 0.0;
        float h = getHeight(id, emission);
        
        // Apply Color Palette to Materials
        vec3 albedo = col2; // Dark Blue Base
        if (r < 0.2) albedo = col1; // Darkest Navy patches
        else if (r > 0.7) albedo = col3 * 0.6; // Slate accents
        
        // Dual Light Setup colored by the palette
        vec3 lightDir1 = normalize(vec3(0.8, 1.0, -0.4));
        vec3 lightCol1 = col5; // Off-White sun
        
        vec3 lightDir2 = normalize(vec3(-0.8, 0.5, 0.4));
        vec3 lightCol2 = col3; // Slate rim light

        // Diffuse
        float diff1 = max(dot(n, lightDir1), 0.0);
        float diff2 = max(dot(n, lightDir2), 0.0);
        
        // Specular
        vec3 ref1 = reflect(-lightDir1, n);
        float spec1 = pow(max(dot(rd, ref1), 0.0), 32.0);
        
        vec3 ref2 = reflect(-lightDir2, n);
        float spec2 = pow(max(dot(rd, ref2), 0.0), 16.0);
        
        float ao = calcAO(warpPos, n);
        
        // Emissive Colors mapped to the new palette
        vec3 emissiveColor = vec3(0.0);
        float freq = clamp(abs(id.x) / 20.0, 0.0, 1.0);
        
        if (freq < 0.2) emissiveColor = col5; // Bass hits are blinding Off-White
        else if (freq < 0.6) emissiveColor = col4; // Mids glow Light Slate
        else emissiveColor = col3; // Treble glows darker Slate

        // Composite lighting
        color = albedo * (diff1 * lightCol1 + diff2 * lightCol2) * ao;
        color += (spec1 * lightCol1 + spec2 * lightCol2) * 0.5 * ao;
        
        // Add Audio Emissive Glow
        if (emission > 0.0) {
            float topGradient = smoothstep(h - 3.0, h + 0.1, warpPos.y);
            color += emissiveColor * emission * topGradient;
            
            if (n.y > 0.5) {
                vec2 localUv = abs(fract(warpPos.xz) - 0.5);
                float edge = step(0.35, max(localUv.x, localUv.y));
                color += emissiveColor * edge * emission * 1.5;
            }
        }
    }
    
    // Add Volumetric Atmosphere 
    float bassGlow = texture(AudioTexture, vec2(0.05, 0.75)).r;
    vec3 atmosColor = mix(col2, col3, sin(Time * 0.2) * 0.5 + 0.5);
    color += atmosColor * glowAcc * (0.5 + bassGlow * 1.2);

    // Fog fading into the darkest palette color
    float fog = exp(-dO * 0.035);
    color = mix(col1 * 0.5, color, fog);

    // Cinematic Tonemapping
    color = color / (1.0 + color);
    color = pow(color, vec3(1.0 / 2.2));

    fragColor = vec4(color, 1.0);
}
