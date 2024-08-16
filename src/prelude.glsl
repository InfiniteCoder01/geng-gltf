#ifndef GLTF_PRELUDE
#define GLTF_PRELUDE

varying vec3 v_world_pos;
varying vec3 v_normal;

varying vec4 v_color;
varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec3 a_normal;

attribute vec4 a_color;
attribute vec2 a_uv;

attribute vec4 a_joints;
attribute vec4 a_weights;

uniform mat4 u_joint_matrices[100];
uniform mat4 u_mesh_matrix;

mat4 skin_matrix() {
    if (a_weights.x == 0.0 && a_weights.y == 0.0 && a_weights.z == 0.0 && a_weights.w == 0.0) {
        return mat4(1.0);
    }
    return a_weights.x * u_joint_matrices[int(a_joints.x)] +
        a_weights.y * u_joint_matrices[int(a_joints.y)] +
        a_weights.z * u_joint_matrices[int(a_joints.z)] +
        a_weights.w * u_joint_matrices[int(a_joints.w)];
}

void copy_outputs() {
    v_color = a_color;
    v_uv = a_uv;
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_base_color;
uniform sampler2D u_base_texture;

vec4 sample_material_texture(vec2 uv) {
    return texture2D(u_base_texture, uv) * u_base_color;
}
#endif

#endif