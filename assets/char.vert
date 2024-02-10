#version 330

in vec2 pos;
in vec2 uv_in;
out vec2 uv;
out vec2 pos_out;

void main() {
	uv = uv_in;
	pos_out = pos;
	vec2 pos_temp = (pos - 0.5) * 2.0;
	pos_temp.y *= -1.0;
	gl_Position = vec4(pos_temp, 0.0, 1.0);
}
