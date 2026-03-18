#version 320 es

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aUV;
layout (location = 2) in vec4 aColor;

out mediump vec2 vUV;
out mediump vec4 vColor;

void main() {
    gl_Position = vec4(aPos, 1.0);
    vUV = aUV;
    vColor = aColor;
}      