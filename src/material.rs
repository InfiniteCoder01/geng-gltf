use super::*;

pub struct Material {
    pub base_color: Rgba<f32>,
    pub base_texture: ugli::Texture,
}

impl Material {
    pub fn load(
        ugli: &Ugli,
        material: gltf::Material,
        buffers: &[gltf::buffer::Data],
        images: &[gltf::image::Data],
    ) -> Result<Self, MeshLoadError> {
        let base_color = Rgba::new(
            material.pbr_metallic_roughness().base_color_factor()[0],
            material.pbr_metallic_roughness().base_color_factor()[1],
            material.pbr_metallic_roughness().base_color_factor()[2],
            material.pbr_metallic_roughness().base_color_factor()[3],
        );

        let base_texture = material
            .pbr_metallic_roughness()
            .base_color_texture()
            .map_or_else(
                || white_texture(ugli),
                |texture| ugli_texture(ugli, texture, images),
            );
        // material.alpha_cutoff()
        // material.alpha_mode()
        // material.double_sided()

        // material.emissive_factor()
        // material.emissive_texture()
        // material.normal_texture()
        // material.occlusion_texture()
        // material.pbr_metallic_roughness()
        Ok(Self {
            base_color,
            base_texture,
        })
    }

    pub fn uniforms(&self) -> impl ugli::Uniforms + '_ {
        ugli::uniforms! {
            u_base_color: self.base_color,
            u_base_texture: &self.base_texture,
        }
    }
}

fn white_texture(ugli: &Ugli) -> ugli::Texture {
    ugli::Texture::new_with(ugli, vec2(1, 1), |_| Rgba::WHITE)
}

fn ugli_texture(
    ugli: &Ugli,
    texture: gltf::texture::Info,
    images: &[gltf::image::Data],
) -> ugli::Texture {
    let image = &images[texture.texture().source().index()];
    let sampler = texture.texture().sampler();
    let (format, r#type) = match image.format {
        gltf::image::Format::R8 => (ugli::Format::R, ugli::Type::UnsignedByte),
        gltf::image::Format::R8G8 => (ugli::Format::RG, ugli::Type::UnsignedByte),
        gltf::image::Format::R8G8B8 => (ugli::Format::RGB, ugli::Type::UnsignedByte),
        gltf::image::Format::R8G8B8A8 => (ugli::Format::RGBA, ugli::Type::UnsignedByte),
        gltf::image::Format::R16 => (ugli::Format::R, ugli::Type::UnsignedShort),
        gltf::image::Format::R16G16 => (ugli::Format::RG, ugli::Type::UnsignedShort),
        gltf::image::Format::R16G16B16 => (ugli::Format::RGB, ugli::Type::UnsignedShort),
        gltf::image::Format::R16G16B16A16 => (ugli::Format::RGBA, ugli::Type::UnsignedByte),
        gltf::image::Format::R32G32B32FLOAT => (ugli::Format::RGB, ugli::Type::Float),
        gltf::image::Format::R32G32B32A32FLOAT => (ugli::Format::RGBA, ugli::Type::Float),
    };

    let mut texture = ugli::Texture::from_raw(
        ugli,
        vec2(image.width as _, image.height as _),
        &image.pixels,
        format,
        r#type,
        false,
    );

    if let Some(filter) = sampler.mag_filter() {
        texture.set_filter(match filter {
            gltf::texture::MagFilter::Nearest => ugli::Filter::Nearest,
            gltf::texture::MagFilter::Linear => ugli::Filter::Linear,
        })
    } else if let Some(filter) = sampler.min_filter() {
        texture.set_filter(match filter {
            gltf::texture::MinFilter::Nearest
            | gltf::texture::MinFilter::NearestMipmapNearest
            | gltf::texture::MinFilter::NearestMipmapLinear => ugli::Filter::Nearest,
            gltf::texture::MinFilter::Linear
            | gltf::texture::MinFilter::LinearMipmapLinear
            | gltf::texture::MinFilter::LinearMipmapNearest => ugli::Filter::Linear,
        })
    }

    let map_wrap_mode = |wrap_mode| match wrap_mode {
        gltf::texture::WrappingMode::ClampToEdge => ugli::WrapMode::Clamp,
        gltf::texture::WrappingMode::MirroredRepeat => ugli::WrapMode::Repeat,
        gltf::texture::WrappingMode::Repeat => ugli::WrapMode::Repeat,
    };
    texture.set_wrap_mode_separate(
        map_wrap_mode(sampler.wrap_s()),
        map_wrap_mode(sampler.wrap_t()),
    );

    texture
}
