#version 450

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec3 v_col;
layout(location = 2) in vec2 v_uv;

void main() {
	gl_Position = vec4(v_pos.x, v_pos.y, v_pos.z, 1.0);
}
