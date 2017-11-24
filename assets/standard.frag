#version 330 core
in vec2 vs_tex;

uniform sampler2D tex_color;

out vec4 fs_color;

void main()
{
  fs_color = texture(tex_color, vs_tex);
}
