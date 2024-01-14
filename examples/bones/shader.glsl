// Derived from: https://github.com/geng-engine/geng/blob/main/examples/gltf/assets/shader.glsl
#include <gltf>

varying vec2 v_uv;
varying vec4 v_color;
varying vec3 v_normal;
varying vec4 v_world_pos;

#ifdef VERTEX_SHADER

uniform mat4 u_projection_matrix; // Comes from geng camera
uniform mat4 u_view_matrix; // Comes from geng camera

void main() {
    // gltf prelude provides attributes a_pos, a_normal, a_uv and a_color, as well as uniform u_model_matrix

    // skin_matrix() returns a matrix to do skinning, based on joints and weights provided for this vertex
    // gltf prelude also provides a_joints, a_weights and u_joint_matrices if you want to do skinning yourself.
    // Some helpful resources:
    // https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_020_Skins.md
    // https://www.youtube.com/watch?v=ZzMnu3v_MOw

    v_world_pos = vec4(a_pos, 1.0) * skin_matrix() * u_model_matrix;
    v_normal = normalize(vec3(u_model_matrix * vec4(a_normal, 0.0)));
    gl_Position = u_projection_matrix * u_view_matrix * v_world_pos;

    v_uv = a_uv;
    v_color = a_color;
}
#endif

#ifdef FRAGMENT_SHADER

uniform vec3 u_light_pos;

void main() {
    // Simple lighting from https://learnopengl.com/Lighting/Basic-Lighting

    vec3 light_color = vec3(1.0, 1.0, 1.0);

    // * Ambient
    float ambient_strength = 0.1;
    vec3 ambient = ambient_strength * light_color;

    // * Diffuse
    vec3 light_dir = normalize(u_light_pos - v_world_pos.xyz / v_world_pos.w);
    float diff = max(dot(v_normal, light_dir), 0.0);
    vec3 diffuse = diff * light_color;

    gl_FragColor = vec4(ambient + diffuse, 1.0) * v_color;
}
#endif