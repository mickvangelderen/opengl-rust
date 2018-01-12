#version 330 core

in vec2 fs_pos_in_tex_space;

out vec4 fs_color;

uniform sampler2D texture_0;

const float dx = 1.0f/800.0f;
const float dy = 1.0f/600.0f;

void main() {
  vec2 offsets[9] = vec2[](
    vec2(-dx, dy),
    vec2(0.0f, dy),
    vec2(dx, dy),
    vec2(-dx, 0.0f),
    vec2(0.0f, 0.0f),
    vec2(dx, 0.0f),
    vec2(-dx, -dy),
    vec2(0.0f, -dy),
    vec2(dx, -dy)
  );

  // float weights[9] = float[](
  //   -1.0f,
  //   -1.0f,
  //   -1.0f,
  //   -1.0f,
  //   9.0f,
  //   -1.0f,
  //   -1.0f,
  //   -1.0f,
  //   -1.0f
  // );

  float weights[9] = float[](
    0.077847f,	0.123317f,	0.077847f,
    0.123317f,	0.195346f,	0.123317f,
    0.077847f,	0.123317f,	0.077847f
  );

  vec4 accumulator = vec4(0.0);
  for (int i = 0; i < 9; i++) {
    accumulator += weights[i] * texture(texture_0, fs_pos_in_tex_space + offsets[i]);
  }

  fs_color = accumulator;
}
