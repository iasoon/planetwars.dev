#extension GL_OES_standard_derivatives : enable
#ifdef GL_ES
precision mediump float;
#endif

varying vec2 v_texCoord;
uniform sampler2D msdf;
uniform vec4 u_bgColor;
uniform vec4 u_fgColor;

uniform float u_distanceRange;
uniform float u_glyphSize;
uniform vec2 u_resolution;
uniform vec4 u_viewbox;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    float scale = u_distanceRange / u_glyphSize * u_resolution.y / u_viewbox.w;

    vec3 msd = texture2D(msdf, v_texCoord).rgb;
    float sd = median(msd.r, msd.g, msd.b);
    float screenPxRange = max(u_distanceRange, scale);
    float screenPxDistance = screenPxRange*(sd - 0.5);
    // float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
    float opacity = clamp(0.0, 1.0, screenPxDistance + 0.5);
    gl_FragColor = vec4(u_fgColor.rgb, u_fgColor.a * opacity);
}