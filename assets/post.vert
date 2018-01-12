#version 330 core

layout (location = 0) in vec2 vs_pos_in_clp_space;
layout (location = 1) in vec2 vs_pos_in_tex_space;

out vec2 fs_pos_in_tex_space;

void main() {
  fs_pos_in_tex_space = vs_pos_in_tex_space;
  gl_Position = vec4(vs_pos_in_clp_space, 0.0, 1.0);
}
