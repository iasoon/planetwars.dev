#ifdef GL_ES
precision mediump float;
#endif

attribute vec2 a_position;
attribute vec2 a_texCoord;

uniform float u_time;

uniform vec4 u_viewbox;         // [x, y, width, height]
uniform vec2 u_resolution;
uniform mat3 u_trans;

varying vec2 v_pos;
varying vec2 v_texCoord;

void main() {
    vec3 pos = vec3(a_position, 1.0);

    pos = u_trans * pos;

    vec2 uv = pos.xy;

    // Viewbox's center is top left, a_position's is in the center to the screen
    // So translate and scale the viewbox**
    uv -= u_viewbox.xy + (u_viewbox.zw * 0.5);
    uv /= u_viewbox.zw * 0.5;

    v_pos = (uv.xy + 1.0) * 0.5;

    gl_Position = vec4(uv.xy, 0.0, 1.0);
    v_texCoord = a_texCoord;
}
