#version 330 core

in vec2 fs_pos_in_tex_space;

out vec4 fs_color;

uniform sampler2D texture_0;
uniform float dx;
uniform float dy;

void main() {
  float weights[25] = float[](
    0.003765f, 0.015019f, 0.023792f, 0.015019f, 0.003765f,
    0.015019f, 0.059912f, 0.094907f, 0.059912f, 0.015019f,
    0.023792f, 0.094907f, 0.150342f, 0.094907f, 0.023792f,
    0.015019f, 0.059912f, 0.094907f, 0.059912f, 0.015019f,
    0.003765f, 0.015019f, 0.023792f, 0.015019f, 0.003765f
  );

  vec4 accumulator = vec4(0.0);
  for (int col = 0; col < 5; col++) {
    for (int row = 0; row < 5; row++) {
      float weight = weights[col * 5 + row];
      vec2 offset = vec2((row - 2)*dx, (col - 2)*dy);
      accumulator += weight * texture(texture_0, fs_pos_in_tex_space + offset);
    }
  }

  fs_color = accumulator;
}
