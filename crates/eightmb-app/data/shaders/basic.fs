#version 320 es

in mediump vec2 vUV;
in mediump vec4 vColor;

out mediump vec4 FragColor;

uniform sampler2D tex;

void main() {
    mediump vec4 sampled = texture(tex, vUV);
    FragColor.rgb = mix(sampled.rgb, vColor.rgb, vColor.a);
    FragColor.a = sampled.a;
}