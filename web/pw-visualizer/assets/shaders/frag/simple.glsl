#ifdef GL_ES
precision mediump float;
#endif

uniform float u_step_interval;
uniform float u_time;
uniform vec3 u_color;
uniform vec3 u_color_next;

void main() {

    vec3 color = mix(u_color, u_color_next, u_time);

    gl_FragColor = vec4(color, 1.0);
}
