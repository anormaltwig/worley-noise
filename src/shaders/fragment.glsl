#version 330 core
precision mediump float;
layout (location = 0) out vec4 color;

uniform vec2 resolution;
uniform float time;
uniform float seed;

float rand(vec2 uv) {
	return fract(sin(dot(uv, vec2(6492.1523 + seed, 152.1234))) * 8364.1612);
}

vec2 rand2f(vec2 uv) {
	return vec2(rand(uv), rand(uv + vec2(0.5)));
}

vec3 worley(vec2 uv, float scale) {
	vec2 scaled_uv = uv * scale;
	vec2 nearest_cell = floor(scaled_uv) + vec2(0.5);

	float dist = 100.f;
	for(int off_x = -1; off_x <= 1; off_x++) {
		for(int off_y = -1; off_y <= 1; off_y++) {
			vec2 target_cell = nearest_cell + vec2(off_x, off_y);
			vec2 target_node = target_cell + rand2f(target_cell) * vec2(sin(time), cos(time));

			dist = min(dist, length(scaled_uv - target_node));
		}
	}
	return vec3(dist);
}

void main() {
	vec2 uv = gl_FragCoord.xy / resolution;

	color = vec4(worley(uv, 8), 1.0);
}
