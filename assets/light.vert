#version 330 core
layout(location = 0) in vec3 in_pos_in_obj_space;

uniform mat4 pos_from_obj_to_clp_space;

void main()
{
  gl_Position = pos_from_obj_to_clp_space*vec4(in_pos_in_obj_space, 1.0);
}
