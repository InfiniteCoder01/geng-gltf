// https://github.com/bwasty/gltf-viewer/blob/master/src/shaders
// https://github.com/geng-engine/geng/blob/main/examples/gltf/assets/shader.glsl
// https://github.com/bwasty/gltf-viewer/blob/master/src/shaders/pbr-frag.glsl

#ifndef GLTF_PBR
#define GLTF_PBR
#include <gltf>

/* ---------------------------------------------------------------------------------------------------- */

#ifdef FRAGMENT_SHADER
const float MIN_ROUGHNESS = 0.04;
const float M_PI = 3.141592653589793;

uniform vec2 u_metallic_roughness;
uniform sampler2D u_metallic_roughness_texture;

uniform sampler2D u_normal_texture;
uniform float u_normal_scale;

struct PBRInfo {
    float NdotL;                  // cos angle between normal and light direction
    float NdotV;                  // cos angle between normal and view direction
    float NdotH;                  // cos angle between normal and half vector
    float LdotH;                  // cos angle between light direction and half vector
    float VdotH;                  // cos angle between view direction and half vector
    float perceptualRoughness;    // roughness value, as authored by the model creator (input to shader)
    float metalness;              // metallic value at the surface
    vec3 reflectance0;            // full reflectance color (normal incidence angle)
    vec3 reflectance90;           // reflectance color at grazing angle
    float alphaRoughness;         // roughness mapped to a more linear change in the roughness (proposed by [2])
    vec3 diffuseColor;            // color contribution from diffuse lighting
    vec3 specularColor;           // color contribution from specular lighting
};

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
vec3 getNormal() {
    vec3 pos_dx = dFdx(v_world_pos);
    vec3 pos_dy = dFdy(v_world_pos);
    vec3 tex_dx = dFdx(vec3(v_uv, 0.0));
    vec3 tex_dy = dFdy(vec3(v_uv, 0.0));
    vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);

    vec3 ng = normalize(v_normal);

    t = normalize(t - ng * dot(ng, t));
    vec3 b = normalize(cross(ng, t));
    mat3 tbn = mat3(t, b, ng);

#ifdef HAS_NORMALMAP
    vec3 n = texture(u_normal_texture, v_uv).rgb;
    n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_normal_scale, u_normal_scale, 1.0)));
#else
    // The tbn matrix is linearly interpolated, so we need to re-normalize
    vec3 n = normalize(tbn[2].xyz);
#endif

    // reverse backface normals
    n *= (2.0 * float(gl_FrontFacing) - 1.0);

    return n;
}

// #ifdef USE_IBL
// // Calculation of the lighting contribution from an optional Image Based Light source.
// // Precomputed Environment Maps are required uniform inputs and are computed as outlined in [1].
// // See our README.md on Environment Maps [3] for additional discussion.
// vec3 getIBLContribution(PBRInfo pbrInputs, vec3 n, vec3 reflection)
// {
//     float mipCount = 9.0; // resolution of 512x512
//     float lod = (pbrInputs.perceptualRoughness * mipCount);
//     // retrieve a scale and bias to F0. See [1], Figure 3
//     vec3 brdf = texture(u_brdfLUT, vec2(pbrInputs.NdotV, 1.0 - pbrInputs.perceptualRoughness)).rgb;
//     vec3 diffuseLight = textureCube(u_DiffuseEnvSampler, n).rgb;

// #ifdef USE_TEX_LOD
//     vec3 specularLight = textureCubeLodEXT(u_SpecularEnvSampler, reflection, lod).rgb;
// #else
//     vec3 specularLight = textureCube(u_SpecularEnvSampler, reflection).rgb;
// #endif

//     vec3 diffuse = diffuseLight * pbrInputs.diffuseColor;
//     vec3 specular = specularLight * (pbrInputs.specularColor * brdf.x + brdf.y);

//     // For presentation, this allows us to disable IBL terms
//     diffuse *= u_ScaleIBLAmbient.x;
//     specular *= u_ScaleIBLAmbient.y;

//     return diffuse + specular;
// }
// #endif

// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
vec3 diffuse(PBRInfo pbrInputs) {
    return pbrInputs.diffuseColor / PI;
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
vec3 specularReflection(PBRInfo pbrInputs) {
    return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
}

// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
float geometricOcclusion(PBRInfo pbrInputs) {
    float NdotL = pbrInputs.NdotL;
    float NdotV = pbrInputs.NdotV;
    float r = pbrInputs.alphaRoughness;

    float attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));
    float attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
    return attenuationL * attenuationV;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
float microfacetDistribution(PBRInfo pbrInputs) {
    float roughnessSq = pbrInputs.alphaRoughness * pbrInputs.alphaRoughness;
    float f = (pbrInputs.NdotH * roughnessSq - pbrInputs.NdotH) * pbrInputs.NdotH + 1.0;
    return roughnessSq / (M_PI * f * f);
}

uniform vec3 u_light_pos;

vec4 pbr(vec3 camera, vec4 base_color) {
    vec2 metallic_roughness = u_metallic_roughness * texture2D(u_metallic_roughness_texture, v_uv).bg;
    float metallic = metallic_roughness.x;
    float perceptualRoughness = metallic_roughness.y;
    metallic = clamp(metallic, 0.0, 1.0);
    perceptualRoughness = clamp(perceptualRoughness, MIN_ROUGHNESS, 1.0);
    float alphaRoughness = perceptualRoughness * perceptualRoughness;

    vec3 f0 = vec3(0.04);
    vec3 diffuseColor = base_color.rgb * (vec3(1.0) - f0);
    diffuseColor *= 1.0 - metallic;
    vec3 specularColor = mix(f0, base_color.rgb, metallic);

    // Compute reflectance.
    float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    vec3 specularEnvironmentR0 = specularColor.rgb;
    vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    vec3 n = getNormal();                        // normal at surface point
    vec3 v = normalize(camera - v_world_pos);    // Vector from surface point to camera
    vec3 l = normalize(/*u_LightDirection*/ u_light_pos - v_world_pos);        // Vector from surface point to light
    vec3 h = normalize(l + v);                   // Half vector between both l and v
    vec3 reflection = -normalize(reflect(v, n));

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
    float NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float LdotH = clamp(dot(l, h), 0.0, 1.0);
    float VdotH = clamp(dot(v, h), 0.0, 1.0);

    PBRInfo pbrInputs = PBRInfo(NdotL, NdotV, NdotH, LdotH, VdotH, perceptualRoughness, metallic, specularEnvironmentR0, specularEnvironmentR90, alphaRoughness, diffuseColor, specularColor);

    // Calculate the shading terms for the microfacet specular shading model
    vec3 F = specularReflection(pbrInputs);
    float G = geometricOcclusion(pbrInputs);
    float D = microfacetDistribution(pbrInputs);

    // Calculation of analytical lighting contribution
    vec3 diffuseContrib = (1.0 - F) * diffuse(pbrInputs);
    vec3 specContrib = F * G * D / (4.0 * NdotL * NdotV);
    vec3 color = NdotL * vec3(1.0, 1.0, 1.0) * (diffuseContrib + specContrib);

//     // Calculate lighting contribution from image based lighting source (IBL)
// #ifdef USE_IBL
//     color += getIBLContribution(pbrInputs, n, reflection);
// #else
//     // Add simple ambient light
//     color += u_AmbientLightColor * u_AmbientLightIntensity * base_color.xyz;
// #endif

//     // Apply optional PBR terms for additional (optional) shading
// #ifdef HAS_OCCLUSIONMAP
//     float ao = texture(u_OcclusionSampler, v_UV[u_OcclusionTexCoord]).r;
//     color = mix(color, color * ao, u_OcclusionStrength);
// #endif

// #ifdef HAS_EMISSIVEMAP
//     vec3 emissive = texture(u_EmissiveSampler, v_UV[u_EmissiveTexCoord]).rgb * u_EmissiveFactor;
//     color += emissive;
// #endif

    // This section uses mix to override final color for reference app visualization
    // of various parameters in the lighting equation.
    // color = mix(color, F, u_ScaleFGDSpec.x);
    // color = mix(color, vec3(G), u_ScaleFGDSpec.y);
    // color = mix(color, vec3(D), u_ScaleFGDSpec.z);
    // color = mix(color, specContrib, u_ScaleFGDSpec.w);

    // color = mix(color, diffuseContrib, u_ScaleDiffBaseMR.x);
    // color = mix(color, base_color.rgb, u_ScaleDiffBaseMR.y);
    // color = mix(color, vec3(metallic), u_ScaleDiffBaseMR.z);
    // color = mix(color, vec3(perceptualRoughness), u_ScaleDiffBaseMR.w);

    return vec4(color, base_color.a);
}
#endif
#endif