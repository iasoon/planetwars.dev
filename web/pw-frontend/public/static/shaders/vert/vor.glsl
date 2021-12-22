#ifdef GL_ES
precision mediump float;
#endif

attribute vec2 a_pos;
attribute vec2 a_center;
attribute float a_own;
attribute float a_intensity;

uniform vec3 u_planet_colours[$PLANETS * 2];
uniform vec4 u_viewbox;         // [x, y, width, height]
uniform vec2 u_resolution;
uniform float u_time;

varying float v_intensity;
varying float v_dist;
varying vec2 v_pos;
varying vec3 v_color;

void main() {
    v_intensity = a_intensity;
    v_dist = distance(a_pos * u_resolution , a_center * u_resolution);

    int own = int(a_own);

    vec2 uv = a_pos;

    // Viewbox's center is top left, a_position's is in the center to the screen
    // So translate and scale the viewbox**
    uv -= u_viewbox.xy + (u_viewbox.zw * 0.5);
    uv /= u_viewbox.zw * 0.5;
    v_pos = uv.xy;

    // v_pos = (uv.xy + 1.0) * 0.5;

    if (own < 0) {
        v_color = vec3(0., 0., 0.);
    } else {
        v_color = mix(u_planet_colours[own * 2], u_planet_colours[own * 2 + 1], u_time);
    }

    gl_Position = vec4(uv.xy, 0.0, 1.0);
}
