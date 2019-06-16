#version 330

#define PI 3.1415927

in vec2 pos;

uniform sampler2D texture;
uniform float time;

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


void main() {
    vec2 skewed_pos = vec2(pos.y * (pos.x * 2.0 - 1.0), time / 5.0) + pos;

    float hue = skewed_pos.x + skewed_pos.y;
    float saturation = 0.85;
    float value = texture2D(texture, skewed_pos).r;

    vec3 sample = hsv2rgb(vec3(hue, saturation, value));

    gl_FragColor = vec4(sample, 1.0);
}
