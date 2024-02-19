#version 330
precision mediump float;

in vec2 uv;
in vec2 pos_out;

out vec4 color;

uniform sampler2D chars_sampler;
uniform sampler1D syspallete_sampler;
uniform sampler1D palletes_sampler;
uniform sampler2D atrtable_sampler;

void main() {
	float color_id = texture(chars_sampler, uv).r * 256.0;
	float pallete_offset = texture(atrtable_sampler, pos_out).r * 256.0;
	color_id += pallete_offset * 4;
	color_id /= 15.0;
	float plcolor_id = texture(palletes_sampler, color_id).r * 256/64;
	color = texture(syspallete_sampler, plcolor_id);
}
