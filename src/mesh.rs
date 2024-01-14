use super::*;

#[derive(Clone, Debug, PartialEq, ugli::Vertex)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_normal: vec3<f32>,
    pub a_uv: vec2<f32>,
    pub a_color: Rgba<f32>,

    pub a_joints: [f32; 4], // A hack.
    pub a_weights: [f32; 4],
}

pub struct Mesh {
    pub data: ugli::VertexBuffer<Vertex>,
    pub material: usize,
    pub mode: ugli::DrawMode,
}

impl Mesh {
    pub fn load(
        ugli: &Ugli,
        primitive: gltf::Primitive,
        buffers: &[gltf::buffer::Data],
        material: usize,
    ) -> Result<Self, MeshLoadError> {
        let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|x| &**x));

        // * Vertices
        // Positions
        let positions = reader
            .read_positions()
            .ok_or(MeshLoadError::MissingPositions)?
            .map(|[x, y, z]| vec3(x, y, z))
            .collect_vec();

        // Normals
        let normals = reader
            .read_normals()
            .map(|normals| normals.map(|[x, y, z]| vec3(x, y, z)).collect_vec());

        // UVs
        let uvs = reader
            .read_tex_coords(0)
            .map(|uvs| uvs.into_f32().map(|[u, v]| vec2(u, v)).collect_vec());

        // Colors
        let colors = reader.read_colors(0).map(|colors| {
            colors
                .into_rgba_f32()
                .map(|[r, g, b, a]| Rgba::new(r, g, b, a))
                .collect_vec()
        });

        // Joints
        let joints = reader.read_joints(0).map(|joints| {
            joints
                .into_u16()
                .map(|joints| joints.map(|joint| joint as f32))
                .collect_vec()
        });

        // Weights
        let weights = reader
            .read_weights(0)
            .map(|weights| weights.into_f32().collect_vec());

        // * Other
        let indices = reader
            .read_indices()
            .ok_or(MeshLoadError::MissingIndices)?
            .into_u32()
            .map(|x| x as usize);

        let mode = match primitive.mode() {
            gltf::mesh::Mode::Points => ugli::DrawMode::Points,
            gltf::mesh::Mode::Lines => ugli::DrawMode::Lines { line_width: 1.0 },
            gltf::mesh::Mode::LineLoop => ugli::DrawMode::LineLoop { line_width: 1.0 },
            gltf::mesh::Mode::LineStrip => ugli::DrawMode::LineStrip { line_width: 1.0 },
            gltf::mesh::Mode::Triangles => ugli::DrawMode::Triangles,
            gltf::mesh::Mode::TriangleStrip => ugli::DrawMode::TriangleStrip,
            gltf::mesh::Mode::TriangleFan => ugli::DrawMode::TriangleFan,
        };

        // * VBO
        let data = ugli::VertexBuffer::new_static(
            ugli,
            indices
                .map(|index| Vertex {
                    a_pos: positions[index],
                    a_normal: normals
                        .as_ref()
                        .map_or(vec3::ZERO, |normals| normals[index]),

                    a_uv: uvs.as_ref().map_or(vec2::ZERO, |uvs| uvs[index]),
                    a_color: colors.as_ref().map_or(Rgba::WHITE, |colors| colors[index]),

                    a_joints: joints.as_ref().map_or([0.0; 4], |joints| joints[index]),
                    a_weights: weights.as_ref().map_or([0.0; 4], |weights| weights[index]),
                })
                .collect(),
        );

        Ok(Self {
            data,
            material,
            mode,
        })
    }
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mesh")
            .field("material", &self.material)
            .field("mode", &self.mode)
            .finish()
    }
}
