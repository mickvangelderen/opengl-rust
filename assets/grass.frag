# version 330 core

struct Material {
  sampler2D diffuse;
  sampler2D normal;
  sampler2D specular;
  float shininess;
};

struct PointLight {
  vec3 pos_in_cam_space;
  vec3 ambient;
  vec3 diffuse;
  vec3 specular;
  float attenuation_constant;
  float attenuation_linear;
  float attenuation_quadratic;
};

#define POINT_LIGHT_LENGTH 4
uniform PointLight point_lights[POINT_LIGHT_LENGTH];
uniform Material material;
uniform mat4 nor_from_obj_to_cam_space;

in vec2 frag_tex;
in vec3 frag_pos_in_cam_space;

out vec4 frag_color;

void main()
{
  // Sample textures.
  // vec3 diffuse_color = texture(material.diffuse, frag_tex).rgb;
  vec3 diffuse_color = vec3(68.0, 88.0, 16.0)/256.0;
  // vec3 specular_color = texture(material.specular, frag_tex).rgb;
  vec3 specular_color = vec3(1.0);

  vec3 frag_nor_in_obj_space = texture(material.normal, frag_tex).rgb;

  vec3 frag_nor_in_cam_space = mat3(nor_from_obj_to_cam_space)*frag_nor_in_obj_space;

  // Accumulate color.
  vec3 color = vec3(0.0);

  for (int i = 0; i < POINT_LIGHT_LENGTH; i++)
  {
    // Compute ambient color component.
    vec3 ambient = point_lights[i].ambient * diffuse_color;

    // Compute diffuse color component.
    vec3 frag_nor_in_cam_space_norm = normalize(frag_nor_in_cam_space);
    vec3 light_dir_in_cam_space_norm = normalize(point_lights[i].pos_in_cam_space - frag_pos_in_cam_space);
    float diffuse_power = max(dot(frag_nor_in_cam_space_norm, light_dir_in_cam_space_norm), 0.0);
    vec3 diffuse = diffuse_power * point_lights[i].diffuse * diffuse_color;

    // Compute specular color component.
    // Since the camera is at 0,0,0 in camera space, this calculation is simplified.
    vec3 view_dir_in_cam_space_norm = normalize(-frag_pos_in_cam_space);
    vec3 reflect_dir_in_cam_space_norm = reflect(-light_dir_in_cam_space_norm, frag_nor_in_cam_space_norm);
    float specular_power = pow(max(dot(view_dir_in_cam_space_norm, reflect_dir_in_cam_space_norm), 0.0), material.shininess);
    vec3 specular = specular_power * point_lights[i].specular * specular_color;

    // Compute attenuation.
    float light_dist = distance(point_lights[i].pos_in_cam_space, frag_pos_in_cam_space);
    float attenuation = 1.0/(
      point_lights[i].attenuation_constant
      + point_lights[i].attenuation_linear * light_dist
      + point_lights[i].attenuation_quadratic * light_dist * light_dist
    );

    color += attenuation*(ambient + diffuse + specular);
  }

  // Combine components.
  frag_color = vec4(color, 1.0);

  // Draw normals.
  // frag_color = vec4((vec3(1.0) + frag_nor_in_cam_space)/2.0, 1.0);
}
