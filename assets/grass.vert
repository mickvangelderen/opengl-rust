#version 330 core

layout (location = 0) in vec3 in_pos_in_obj_space;
layout (location = 1) in vec2 in_tex;
layout (location = 2) in vec3 in_nor_in_obj_space;

out vec2 frag_tex;
out vec3 frag_pos_in_cam_space;
// out vec3 frag_nor_in_cam_space; // use normal map instead.

uniform mat4 pos_from_obj_to_cam_space;
// uniform mat4 nor_from_obj_to_cam_space;
uniform mat4 pos_from_obj_to_clp_space;

void main()
{
  frag_tex = in_tex;
  // frag_nor_in_cam_space = mat3(nor_from_obj_to_cam_space)*in_nor_in_obj_space;
  frag_pos_in_cam_space = (pos_from_obj_to_cam_space*vec4(in_pos_in_obj_space, 1.0)).xyz;
  gl_Position = pos_from_obj_to_clp_space*vec4(in_pos_in_obj_space, 1.0);
}
