#version 330 core
layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_color;
layout (location = 2) in vec2 in_tex;

out vec3 vs_color;
out vec2 vs_tex;

uniform mat4 cam_to_obj;
uniform mat4 projection;

void main()
{
  vs_color = in_color;
  vs_tex = in_tex;
  gl_Position = projection*cam_to_obj*vec4(in_pos, 1.0);
}
