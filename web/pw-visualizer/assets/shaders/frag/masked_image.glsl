#ifdef GL_ES
precision mediump float;
#endif

// Passed in from the vertex shader.
varying vec2 v_texCoord;

uniform float u_step_interval;
uniform float u_time;
uniform vec3 u_color;
uniform vec3 u_color_next;


// The texture.
uniform sampler2D u_texture;

void main() {
   float alpha = texture2D(u_texture, v_texCoord).a;
   vec3 color = mix(u_color, u_color_next, u_time);
   gl_FragColor = vec4(color, alpha);
}
