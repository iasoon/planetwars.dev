#ifdef GL_ES
precision mediump float;
#endif

// Passed in from the vertex shader.
varying vec2 v_texCoord;

// The texture.
uniform sampler2D u_texture;

void main() {
   gl_FragColor = texture2D(u_texture, v_texCoord);
//    gl_FragColor = vec4(0.7, 0.7, 0.0, 1.0);
}
