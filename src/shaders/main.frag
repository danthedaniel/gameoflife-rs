#version 330

#define PI 3.1415927

in vec2 pos;

uniform sampler2D texture;
uniform float time;
uniform vec2 board_size;
uniform vec2 window_size;

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

    vec2 aspect_adjusted_post = pos * vec2(window_size.x / window_size.y, 1.0);

    float adjusted_time = time / 10.0;

    // Rotate around center
    float rotation = sin(adjusted_time);
    vec2 skewed_pos = rotate(aspect_adjusted_post - center, rotation) + center;
    skewed_pos += vec2(cos(adjusted_time), sin(adjusted_time));

    // Zoom around center
    float zoom = 0.7 + (sin(adjusted_time) / 3.0);
    vec2 zoomed_pos = (skewed_pos - center) * zoom + center;

    // Rainbow circular hue shift
    vec4 tex_sample = texture2D(texture, zoomed_pos);
    float hue = distance(quant(zoomed_pos, board_size), center) * 4.0 + (time / 5.0) / 2.0;
    float saturation = 1.0 - clamp((tex_sample.w - (1.0 / 255.0)) * 2.0, 0.0, 1.0);
    float value = tex_sample.r;

    vec4 shadow_sample = texture2D(texture, zoomed_pos + rotate(vec2(-0.002, 0.004), rotation * 0.8));
    float shadow = shadow_sample.r;

    vec3 out_sample = vec3(0.0);
    
    if (shadow > 0.0) {
        out_sample = vec3(0.1);
    }
    
    if (value > 0.0) {
        out_sample = hsv2rgb(vec3(hue, saturation, value));
    }

    gl_FragColor = vec4(out_sample, 1.0);
}
