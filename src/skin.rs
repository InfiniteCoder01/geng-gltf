use super::*;

#[derive(Clone, Debug)]
pub struct Joint {
    pub node_index: usize,
    pub inverse_bind_matrix: mat4<f32>,
}

#[derive(Clone, Debug)]
pub struct Skin {
    pub joints: Vec<Joint>,
}

impl Skin {
    pub fn load(
        skin: gltf::Skin<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Self, MeshLoadError> {
        let reader = skin.reader(|buffer| buffers.get(buffer.index()).map(|x| &**x));
        let inverse_bind_matrices = reader
            .read_inverse_bind_matrices()
            .ok_or(MeshLoadError::MissingInverseBindMatrices)?;

        let joints = std::iter::zip(inverse_bind_matrices, skin.joints())
            .map(|(inverse_bind_matrix, joint)| Joint {
                node_index: joint.index(),
                inverse_bind_matrix: mat4::new(inverse_bind_matrix),
            })
            .collect_vec();

        Ok(Self { joints })
    }

    pub fn uniforms(&self, node_transforms: &[mat4<f32>]) -> impl ugli::Uniforms {
        let mut transforms = Vec::with_capacity(self.joints.len());
        for joint in self.joints.iter() {
            transforms.push(node_transforms[joint.node_index] * joint.inverse_bind_matrix);
        }

        ugli::SingleUniform::new("u_joint_matrices[0]", transforms)
    }
}

impl Model {
    pub fn armature_uniforms(&self, node_transforms: &[mat4<f32>]) -> impl ugli::Uniforms {
        self.skins
            .iter()
            .map(|skin| skin.uniforms(node_transforms))
            .collect_vec()
    }
}
