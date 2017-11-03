#version 330 core
in vec3 vs_color;
in vec2 vs_tex;

uniform float mix_val;
uniform sampler2D tex_color;

out vec4 fs_color;

void main()
{
  fs_color = mix(
    texture(tex_color, vs_tex),
    vec4(vs_color, 1.0f),
    mix_val
  );
}
