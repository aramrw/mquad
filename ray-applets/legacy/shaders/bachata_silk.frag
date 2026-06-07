#version 150
precision mediump float;

uniform vec2 Resolution;
uniform float Time;
uniform sampler2D AudioTexture;

in vec2 uv;
out vec4 fragColor;

const int MAX_STEPS = 100;
const float MAX_DIST = 50.0;
const float SURF_DIST = 0.005;

mat2 rot(float a) {
    float s = sin(a), c = cos(a);
    return mat2(c, -s, s, c);
}

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float GetDist(vec3 p) {
    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float mids = texture(AudioTexture, vec2(0.4, 0.75)).r;

    // Sensual, sweeping waves like silk or a flowing dress
    vec3 q = p;
    
    // Gentle twisting of the fabric based on time and depth
    q.xz *= rot(sin(p.z * 0.2 + Time * 0.5) * 0.5);
    
    // Smooth, rolling hills of silk
    float wave1 = sin(q.x * 0.8 + Time * 1.2 + q.z * 0.4) * 1.2;
    float wave2 = cos(q.x * 1.5 - Time * 0.8 + q.z * 0.9) * 0.6;
    
    // High frequency micro-ripples driven heavily by the guitar/bongos (mids)
    float micro = sin(q.x * 5.0 + q.z * 4.0 - Time * 3.0) * (0.1 + mids * 0.8);
    
    // Bass makes the waves swell up passionately
    float height = wave1 + wave2 + micro;
    
    // Extreme bass pop to make the silk leap up on the heavy beats
    float bassPop = bass * 2.0;
    if (bass > 0.6) {
        bassPop += pow(bass - 0.6, 1.5) * 15.0;
    }
    
    float d = p.y + 1.5 - height * (0.6 + bassPop);
    
    // Scale down for safety with heavy math distortion so the raymarcher doesn't clip
    return d * 0.4; 
}

vec3 GetNormal(vec3 p) {
    float d = GetDist(p);
    vec2 e = vec2(0.01, 0);
    vec3 n = d - vec3(
        GetDist(p - e.xyy),
        GetDist(p - e.yxy),
        GetDist(p - e.yyx)
    );
    return normalize(n);
}

void main() {
    vec2 p = (uv.xy * 2.0 - 1.0);
    p.x *= Resolution.x / Resolution.y;

    float bass = texture(AudioTexture, vec2(0.05, 0.75)).r;
    float mids = texture(AudioTexture, vec2(0.4, 0.75)).r;
    float treble = texture(AudioTexture, vec2(0.9, 0.75)).r;

    // Camera: Swaying side to side like the bachata basic step
    // The sway speed and width increases with the mids (guitars/bongos)
    float swaySway = sin(Time * 1.5) * (2.0 + mids * 2.0);
    float swayX = swaySway;
    
    // The dip in the dance step gets deeper when the bass hits hard
    float swayY = cos(Time * 3.0) * (0.1 + bass * 0.4); 
    
    // Forward movement pulses slightly with the music rather than just being linear
    float forwardMotion = Time * 3.0 + sin(Time * 2.0) * bass * 1.5;
    
    vec3 ro = vec3(swayX, 2.0 + swayY, forwardMotion);
    vec3 lookAt = ro + vec3(sin(Time * 0.75) * (1.0 + mids), -1.0, 4.0);
    
    vec3 forward = normalize(lookAt - ro);
    vec3 right = normalize(cross(vec3(0.0, 1.0, 0.0), forward));
    vec3 up = normalize(cross(right, forward));
    
    // Camera roll becomes much more dramatic when the music swells
    float roll = sin(Time * 1.2) * (0.1 + bass * 0.25);
    vec2 pRot = p * rot(roll);
    
    vec3 rd = normalize(forward + right * pRot.x + up * pRot.y);

    float dO = 0.0;
    vec3 pHit;
    
    for(int i = 0; i < MAX_STEPS; i++) {
        pHit = ro + rd * dO;
        float dS = GetDist(pHit);
        dO += dS;
        if(dO > MAX_DIST || abs(dS) < SURF_DIST) break;
    }

    // 1. Draw the Sky / Background First
    // Deep romantic night sky
    vec3 skyColor = vec3(0.02, 0.0, 0.05); 
    
    // Sweeping club lights / auroras in the sky reacting to the music
    float aurora1 = smoothstep(0.0, 1.0, sin(rd.x * 5.0 + Time) * cos(rd.y * 3.0 - Time * 0.5));
    float aurora2 = smoothstep(0.0, 1.0, sin(rd.x * 3.0 - Time * 1.2) * cos(rd.y * 4.0 + Time * 0.8));
    
    skyColor += vec3(0.6, 0.0, 0.1) * aurora1 * bass * 2.0;   // Deep red sweeps with bass
    skyColor += vec3(0.3, 0.0, 0.5) * aurora2 * mids * 1.5;   // Purple sweeps with mids
    
    // Floating gold embers / stars reacting to treble
    // Offset by Time so the stars slowly drift across the sky
    float starHash = hash(rd.xy * 150.0 + vec2(Time * 0.02, -Time * 0.05));
    float stars = smoothstep(0.985, 1.0, starHash);
    skyColor += vec3(1.0, 0.8, 0.4) * stars * (1.0 + treble * 8.0);
    
    vec3 color = skyColor;
    
    // 2. Draw the Silk Ground
    if(dO < MAX_DIST) {
        vec3 n = GetNormal(pHit);
        
        vec3 colDeep = vec3(0.08, 0.0, 0.12); // Deep club purple shadow
        vec3 colRed = vec3(0.85, 0.05, 0.15); // Vibrant crimson satin
        vec3 colGold = vec3(1.0, 0.85, 0.3);  // Bright gold chain highlights
        
        // Mix color based on height of the wave and bass energy
        float heightNorm = clamp((pHit.y + 1.5) / 3.0, 0.0, 1.0);
        vec3 albedo = mix(colDeep, colRed, heightNorm + bass * 0.8);
        
        // Lighting - sultry spotlight from above/forward
        vec3 lightPos = ro + vec3(0.0, 5.0, 5.0);
        vec3 lightDir = normalize(lightPos - pHit);
        
        float diff = max(dot(n, lightDir), 0.0);
        
        // High gloss specular for that smooth satin dress look
        vec3 ref = reflect(-lightDir, n);
        float spec = pow(max(dot(rd, ref), 0.0), 64.0);
        
        // Treble (vocals, güira) flashes the gold specular shine
        float specPower = 1.0 + treble * 10.0;
        
        // Rim light (ambient club lighting from behind)
        float rim = 1.0 - max(dot(n, -rd), 0.0);
        rim = smoothstep(0.5, 1.0, rim);
        vec3 rimColor = vec3(0.8, 0.0, 0.8); // Bright purple rim
        
        vec3 groundColor = albedo * diff;
        groundColor += colGold * spec * specPower; // Glossy shine
        groundColor += rimColor * rim * (0.4 + bass * 0.8); // Edges glow
        
        // Volumetric mist in the valleys blending into the sky
        float fog = exp(-dO * 0.03); // Softer fog transition
        vec3 mistColor = vec3(0.1, 0.0, 0.1) + vec3(0.4, 0.0, 0.1) * bass;
        
        // Blend ground with mist, then mix into the sky background based on distance
        groundColor = mix(mistColor, groundColor, fog);
        color = mix(skyColor, groundColor, fog);
    }
    
    // Cinematic Vignette
    color *= 1.0 - dot(p, p) * 0.35;

    // Tonemapping for rich colors
    color = color / (1.0 + color);
    color = pow(color, vec3(1.0 / 2.2));

    fragColor = vec4(color, 1.0);
}
