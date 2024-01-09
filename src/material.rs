use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material {}

impl Material {
    pub fn uniforms(&self) -> impl ugli::Uniforms + '_ {
        ugli::uniforms! {}
    }
}
