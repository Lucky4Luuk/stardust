#version 330

layout(location = 0) in vec3 aPos;

void main() {
	gl_Position = vec4(aPos.x * 0.5, aPos.y * 0.5, 0.0, 1.0);
}
