#version 300 es

in vec2 pos;
in vec2 uv_in;
out vec2 uv;

void main() {
	uv = uv_in;
	vec2 pos_temp = (pos - 0.5) * 2.0;
	pos_temp.y *= -1.0;
	gl_Position = vec4(pos_temp, 0.0, 1.0);
}
