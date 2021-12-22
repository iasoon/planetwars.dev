#ifdef GL_ES
precision mediump float;
#endif

#define PI 3.141592

uniform float u_step_interval;
uniform float u_time;
uniform bool u_vor;

varying float v_intensity;
varying float v_dist;
varying vec3 v_color;
varying vec2 v_pos;

void main() {
    gl_FragColor = vec4(v_color, (1.0 - pow(1.0 - v_intensity, 1.23)) * 0.7);
}
