use geng::prelude::{itertools::Itertools, *};

pub use animation::*;
pub use camera::*;
pub use material::*;
pub use mesh::*;
pub use skin::*;

mod animation;
mod camera;
mod material;
mod mesh;
mod skin;

pub struct Model {
    pub document: gltf::Document,

    pub cameras: Vec<Projection>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub skins: Vec<Skin>,
    pub animations: HashMap<String, Animation>,

    pub transform: mat4<f32>,
    pub transforms: Vec<mat4<f32>>,
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
        (document, buffers, images): (
            gltf::Document,
            Vec<gltf::buffer::Data>,
            Vec<gltf::image::Data>,
        ),
    ) -> Result<Self, MeshLoadError> {
        if document.default_scene().is_none() {
            return Err(MeshLoadError::NoDefaultScene);
        }

        debug_node_tree(document.default_scene().unwrap().nodes());

        let mut materials = Vec::new();
        for material in document.materials() {
            materials.push(Material::load(ugli, material, &buffers, &images)?);
        }

        let mut meshes = Vec::new();
        for mesh in document.meshes() {
            log::trace!("Loading mesh {:?}", mesh.name());
            for primitive in mesh.primitives() {
                let material = match primitive.material().index() {
                    Some(index) => index,
                    None => {
                        materials.push(Material::load(
                            ugli,
                            primitive.material(),
                            &buffers,
                            &images,
                        )?);
                        materials.len() - 1
                    }
                };
                meshes.push(Mesh::load(ugli, primitive, &buffers, material)?);
            }
        }

        let mut cameras = Vec::new();
        for camera in document.cameras() {
            cameras.push(Projection::from(camera.projection()));
        }

        let mut skins = Vec::new();
        for skin in document.skins() {
            skins.push(Skin::load(skin, &buffers)?);
        }

        let mut animations = HashMap::new();
        for animation in document.animations() {
            if let Some(name) = animation.name() {
                animations.insert(name.to_owned(), Animation::load(animation, &buffers)?);
            } else {
                log::warn!(
                    "Unnamed animations are not supported yet. Skipping animation id {}",
                    animation.index()
                );
            }
        }

        let transforms = vec![mat4::identity(); document.nodes().count()];

        Ok(Self {
            document,

            cameras,
            meshes,
            materials,
            skins,
            animations,

            transform: mat4::identity(),
            transforms,
        })
    }

    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        program: &ugli::Program,
        camera: Option<&str>,
        uniforms: impl ugli::Uniforms,
        draw_parameters: impl std::borrow::Borrow<ugli::DrawParameters>,
    ) {
        let draw_parameters = draw_parameters.borrow();

        struct Transforms<'a> {
            node: Vec<mat4<f32>>,
            model: Vec<mat4<f32>>,
            camera: Vec<mat4<f32>>,
            camera_name: Option<&'a str>,
            camera_index: Option<usize>,
        }

        let mut transforms = Transforms {
            node: vec![mat4::identity(); self.transforms.len()],
            model: vec![mat4::identity(); self.meshes.len()],
            camera: vec![mat4::identity(); self.cameras.len()],
            camera_name: camera,
            camera_index: None,
        };

        fn traverse(
            node: gltf::Node,
            parent_transform: mat4<f32>,
            model: &Model,
            transforms: &mut Transforms,
        ) {
            transforms.node[node.index()] = mat4::new(node.transform().matrix())
                * parent_transform
                * model.transforms[node.index()];

            if let Some(mesh) = node.mesh() {
                transforms.model[mesh.index()] = transforms.node[node.index()];
            }

            if let Some(camera) = node.camera() {
                transforms.camera[camera.index()] = transforms.node[node.index()];
                if camera.name() == transforms.camera_name {
                    transforms.camera_index = Some(camera.index());
                }
            }

            for child in node.children() {
                traverse(child, transforms.node[node.index()], model, transforms);
            }
        }

        for node in self.document.default_scene().unwrap().nodes() {
            traverse(node, self.transform, self, &mut transforms);
        }

        if let Some(camera) = camera {
            if transforms.camera_index.is_none() {
                log::error!("Camera {:?} not found!", camera);
            }
        }

        for (index, mesh) in self.meshes.iter().enumerate() {
            ugli::draw(
                framebuffer,
                program,
                mesh.mode,
                &mesh.data,
                (
                    (
                        if let Some(camera) = transforms.camera_index {
                            vec![geng::camera::Uniforms3d {
                                u_projection_matrix: self.cameras[camera]
                                    .matrix(framebuffer.size().map(|x| x as f32)),
                                u_view_matrix: transforms.camera[camera].transpose().inverse(),
                            }]
                        } else {
                            Vec::new()
                        },
                        ugli::SingleUniform::new("u_model_matrix", transforms.model[index]),
                    ),
                    (
                        self.armature_uniforms(&transforms.node),
                        self.materials[mesh.material].uniforms(),
                        &uniforms,
                    ),
                ),
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
    #[error("Missing inverse bind matrices")]
    MissingInverseBindMatrices,
    #[error("No default scene, multiple scenes are not supported yet")]
    NoDefaultScene,
    #[error("Missing animation inputs (time)")]
    MissingAnimationInputs,
    #[error("Missing animation outputs")]
    MissingAnimationOutputs,
}

pub fn debug_node_tree<'a>(nodes: impl Iterator<Item = gltf::Node<'a>>) {
    fn traverse(node: gltf::Node, indent: usize) {
        println!(
            "{}{:?} #{} ({})",
            "  ".repeat(indent),
            node.name(),
            node.index(),
            [
                node.camera().map(|_| "Camera"),
                node.mesh().map(|_| "Mesh"),
                node.skin().map(|_| "Skin")
            ]
            .into_iter()
            .flatten()
            .collect_vec()
            .join(", "),
        );
        for child in node.children() {
            traverse(child, indent + 1);
        }
    }

    for node in nodes {
        traverse(node, 0);
    }
}

pub fn prelude_shader() -> String {
    include_str!("prelude.glsl").to_owned()
}

pub fn pbr_shader() -> String {
    include_str!("pbr.glsl").to_owned()
}
