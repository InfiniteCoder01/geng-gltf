use geng::prelude::{itertools::Itertools, *};

pub use material::*;
pub use mesh::*;

mod material;
mod mesh;

pub struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn load(ugli: &Ugli, path: impl AsRef<std::path::Path>) -> Result<Self, MeshLoadError> {
        let gltf = gltf::import(path)?;
        Self::from_gltf(ugli, gltf)
    }

    pub fn from_slice(ugli: &Ugli, bytes: impl AsRef<[u8]>) -> Result<Self, MeshLoadError> {
        let gltf = gltf::import_slice(bytes)?;
        Self::from_gltf(ugli, gltf)
    }

    pub fn from_gltf(
        ugli: &Ugli,
        (document, buffers, _images): (
            gltf::Document,
            Vec<gltf::buffer::Data>,
            Vec<gltf::image::Data>,
        ),
    ) -> Result<Self, MeshLoadError> {
        let mut meshes = Vec::new();
        for mesh in document.meshes() {
            log::trace!("Loading mesh {:?}", mesh.name());
            for primitive in mesh.primitives() {
                meshes.push(Mesh::load(ugli, primitive, &buffers)?);
            }
        }

        Ok(Self { meshes })
    }

    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        program: &ugli::Program,
        uniforms: impl ugli::Uniforms,
        draw_parameters: impl std::borrow::Borrow<ugli::DrawParameters>,
    ) {
        let draw_parameters = draw_parameters.borrow();
        for mesh in &self.meshes {
            ugli::draw(
                framebuffer,
                program,
                mesh.mode,
                &mesh.data,
                (mesh.material.uniforms(), &uniforms),
                draw_parameters,
            );
        }
    }
}

impl geng::asset::Load for Model {
    type Options = ();
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let path = path.to_owned();
        let ugli = manager.ugli().clone();
        async move {
            Self::from_slice(&ugli, file::load_bytes(&path).await?).map_err(|err| anyhow!(err))
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("glb");
}

// * ------------------------------------ Utility ----------------------------------- * //
#[derive(thiserror::Error, Debug)]
pub enum MeshLoadError {
    #[error(transparent)]
    GltfError(#[from] gltf::Error),
    #[error("Missing positions")]
    MissingPositions,
    #[error("Missing indices, this is not supported yet")]
    MissingIndices,
}
