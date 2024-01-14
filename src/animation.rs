use super::*;

pub enum Outputs {
    Translation(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
    Scale(Vec<[f32; 3]>),
}

pub struct Channel {
    target: usize,
    inputs: Vec<f32>,
    outputs: Outputs,
}

pub struct Animation {
    pub channels: Vec<Channel>,
}

impl Animation {
    pub fn load(
        animation: gltf::Animation<'_>,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Self, MeshLoadError> {
        let mut channels = Vec::new();
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| buffers.get(buffer.index()).map(|x| &**x));
            let inputs = reader
                .read_inputs()
                .ok_or(MeshLoadError::MissingAnimationInputs)?
                .collect();
            let outputs = reader
                .read_outputs()
                .ok_or(MeshLoadError::MissingAnimationOutputs)?;

            use gltf::animation::util::ReadOutputs;
            let outputs = match outputs {
                ReadOutputs::Translations(translations) => {
                    Outputs::Translation(translations.collect())
                }
                ReadOutputs::Rotations(rotations) => {
                    Outputs::Rotation(rotations.into_f32().collect())
                }
                ReadOutputs::Scales(scales) => Outputs::Scale(scales.collect()),
                ReadOutputs::MorphTargetWeights(_) => todo!("Morph targets"),
            };

            channels.push(Channel {
                target: channel.target().node().index(),
                inputs,
                outputs,
            });
        }
        Ok(Self { channels })
    }
}

impl Model {
    pub fn reset_transforms(&mut self) {
        self.transforms.fill(mat4::identity());
    }

    pub fn apply_animation(&mut self, name: &str, time: f32) {
        let Some(animation) = self.animations.get(name) else {
            log::error!("Animation {:?} not found", name);
            return;
        };

        for channel in &animation.channels {
            let index = match channel.inputs.binary_search_by(|t| t.total_cmp(&time)) {
                Ok(index) => index,
                Err(index) => index.saturating_sub(1),
            };
            let transform = mat4::new(
                match &channel.outputs {
                    Outputs::Translation(translations) => gltf::scene::Transform::Decomposed {
                        translation: translations[index],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                    Outputs::Rotation(rotations) => gltf::scene::Transform::Decomposed {
                        translation: [0.0, 0.0, 0.0],
                        rotation: rotations[index],
                        scale: [1.0, 1.0, 1.0],
                    },
                    Outputs::Scale(scales) => gltf::scene::Transform::Decomposed {
                        translation: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: scales[index],
                    },
                }
                .matrix(),
            );

            self.transforms[channel.target] = transform * self.transforms[channel.target];
        }
    }
}
