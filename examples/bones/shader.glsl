// Derived from: https://github.com/geng-engine/geng/blob/main/examples/gltf/assets/shader.glsl
#include <gltf>
#include <gltf-pbr>

varying vec3 camera;

#ifdef VERTEX_SHADER
uniform mat4 u_projection_matrix; // Comes from geng camera
uniform mat4 u_view_matrix; // Comes from geng camera

void main() {
    // <gltf> prelude header provides v_world_pos and v_normal that you have to set
    // As well as a_pos, u_mesh_matrix and skin_matrix() to compute it.
    vec4 world_pos = vec4(a_pos, 1.0) * skin_matrix() * u_mesh_matrix;
    v_world_pos = world_pos.xyz / world_pos.w;
    v_normal = normalize(vec3(u_mesh_matrix * vec4(a_normal, 0.0)));

    gl_Position = u_projection_matrix * u_view_matrix * world_pos;

    camera = (vec4(0.0, 0.0, 0.0, 0.0) * inverse(u_view_matrix)).xyz;

    // <gltf> header provides a handy function to copy color and uv attributes to fragment shader,
    // but you can also do it yourself by setting v_color and v_uv to the corresponding a_color and a_uv.
    copy_outputs();
}
#endif

#ifdef FRAGMENT_SHADER

// uniform vec3 u_light_pos;

void main() {
    gl_FragColor = pbr(camera, sample_material_texture(v_uv) * v_color);
}
#endif