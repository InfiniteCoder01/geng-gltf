// https://github.com/geng-engine/geng/blob/main/examples/gltf/assets/shader.glsl

varying vec2 v_uv;
varying vec4 v_color;
varying vec3 v_normal;
varying vec4 v_world_pos;

#ifdef VERTEX_SHADER

attribute vec3 a_pos;
attribute vec3 a_normal;

attribute vec2 a_uv;
attribute vec4 a_color;

attribute vec4 a_joints;
attribute vec4 a_weights;

uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_model_matrix;

// https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_020_Skins.md
// Helpful: https://www.youtube.com/watch?v=ZzMnu3v_MOw
uniform mat4 u_joint_mat[2];

void main() {
    mat4 skin_matrix = a_weights.x * u_joint_mat[int(a_joints.x)] +
        a_weights.y * u_joint_mat[int(a_joints.y)] +
        a_weights.z * u_joint_mat[int(a_joints.z)] +
        a_weights.w * u_joint_mat[int(a_joints.w)];

    v_world_pos = vec4(a_pos, 1.0) * u_model_matrix;
    v_normal = normalize(vec3(u_model_matrix * vec4(a_normal, 0.0)));
    gl_Position = u_projection_matrix * u_view_matrix * v_world_pos;

    v_uv = a_uv;
    v_color = vec4(u_joint_mat[0][0][0], 1.0, 1.0, 1.0); // a_color;
}
#endif

#ifdef FRAGMENT_SHADER

uniform vec3 u_light_pos;

void main() {
    // https://learnopengl.com/Lighting/Basic-Lighting

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