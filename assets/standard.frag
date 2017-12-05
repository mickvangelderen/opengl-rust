#version 330 core
struct Material {
  sampler2D diffuse;
  sampler2D specular;
  float shininess;
};

struct Light {
  vec3 pos_in_cam_space;
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
};

in vec3 vs_pos_in_cam_space;
in vec2 vs_tex;
in vec3 vs_nor_in_cam_space;

uniform Light light;
uniform Material material;

out vec4 fs_color;

void main()
{
  // Sample textures.
  vec3 diffuse_color = texture(material.diffuse, vs_tex).rgb;
  vec3 specular_color = texture(material.specular, vs_tex).rgb;

  // Compute ambient color component.
  vec3 ambient = light.ambient * diffuse_color;

  // Compute diffuse color component.
  vec3 vs_nor_in_cam_space_norm = normalize(vs_nor_in_cam_space);
  vec3 light_dir_in_cam_space_norm = normalize(light.pos_in_cam_space - vs_pos_in_cam_space);
  float diffuse_power = max(dot(vs_nor_in_cam_space_norm, light_dir_in_cam_space_norm), 0.0);
  vec3 diffuse = diffuse_power * light.diffuse * diffuse_color;

  // Compute specular color component.
  // Since the camera is at 0,0,0 in camera space, this calculation is simplified.
  vec3 view_dir_in_cam_space_norm = normalize(-vs_pos_in_cam_space);
  vec3 reflect_dir_in_cam_space_norm = reflect(-light_dir_in_cam_space_norm, vs_nor_in_cam_space_norm);
  float specular_power = pow(max(dot(view_dir_in_cam_space_norm, reflect_dir_in_cam_space_norm), 0.0), material.shininess);
  vec3 specular = specular_power * light.specular * specular_color;

  // Combine components.
  fs_color = vec4(ambient + diffuse + specular, 1.0);

  // Draw normals.
  // fs_color = vec4(0.5*(vec3(1.0) + vs_nor), 1.0);
}
