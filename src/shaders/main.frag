#version 330

#define PI 3.1415927

in vec2 pos;

uniform sampler2D texture;
uniform float time;
uniform vec2 board_size;

vec2 rotate(vec2 vec, float angle) {
    float s = sin(angle);
    float c = cos(angle);

    mat2 rotationMat = mat2(
        c, -s,
        s,  c
    );

    return rotationMat * vec;
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float quant(float val, float n) {
    return floor(val * n) / n;
}

vec2 quant(vec2 val, vec2 n) {
    return floor(val * n) / n;
}

void main() {
    vec2 center = vec2(0.5);
    vec2 skewed_pos = rotate(pos - center, sin(time)) + center;

    float hue = distance(quant(skewed_pos, board_size), center) * 4.0 + time / 2.0;
    float saturation = 0.95;
    float value = texture2D(texture, skewed_pos).r;

    float shadow = texture2D(texture, skewed_pos + vec2(-0.003, 0.007)).r;

    vec3 sample = vec3(0.0);
    
    if (shadow > 0.0) {
        sample = vec3(0.1);
    }
    
    if (value > 0.0) {
        sample = hsv2rgb(vec3(hue, saturation, value));
    }

    gl_FragColor = vec4(sample, 1.0);
}
