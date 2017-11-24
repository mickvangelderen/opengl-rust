#version 330 core
in vec3 vs_pos_in_cam_space;
in vec2 vs_tex;
in vec3 vs_nor_in_cam_space;

uniform sampler2D tex_color;
uniform vec3 light_pos_in_cam_space;

out vec4 fs_color;

void main()
{
  vec3 vs_nor_in_cam_space_norm = normalize(vs_nor_in_cam_space);

  vec3 light_dir_in_cam_space_norm = normalize(light_pos_in_cam_space - vs_pos_in_cam_space);

  // Since the camera is at 0,0,0 in camera space, this calculation is simplified.
  vec3 view_dir_in_cam_space_norm = normalize(-vs_pos_in_cam_space);

  // TODO: Verify with math that reflect when passed two normalized vectors actually returns a normalized vector.
  vec3 reflect_dir_in_cam_space_norm = reflect(-light_dir_in_cam_space_norm, vs_nor_in_cam_space_norm);

  float s = pow(max(dot(view_dir_in_cam_space_norm, reflect_dir_in_cam_space_norm), 0.0), 32);

  // TODO: Let the light color affect the spotlight color?
  // vec3 specular = specularStrength * spec * lightColor;

  float a = 0.2;
  float d = max(dot(vs_nor_in_cam_space_norm, light_dir_in_cam_space_norm), 0.0);
  fs_color = (a + d + s)*texture(tex_color, vs_tex);
  // Draw normals.
  // fs_color = vec4(0.5*(vec3(1.0) + vs_nor), 1.0);
}
