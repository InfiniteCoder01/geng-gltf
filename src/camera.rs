use super::*;

/// Projection object is used inside the node tree
/// to represent the camera controlled by the scene/node tree.
/// It's also used inside the [FreeCamera] object,
/// which is recomended in cases, where you need to control the camera manually (e. g. Games).
pub enum Projection {
    Orthographic {
        x_mag: f32,
        y_mag: f32,
        z_near: f32,
        z_far: f32,
    },
    Perspective {
        aspect_ratio: Option<f32>,
        y_fov: f32,
        z_near: f32,
        z_far: f32,
    },
}

impl Projection {
    pub fn matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        match *self {
            Projection::Orthographic {
                x_mag,
                y_mag,
                z_near,
                z_far,
            } => mat4::frustum(
                -0.5 * x_mag,
                0.5 * x_mag,
                -0.5 * y_mag,
                0.5 * y_mag,
                z_near,
                z_far,
            ),
            Projection::Perspective {
                aspect_ratio: _,
                y_fov,
                z_near,
                z_far,
            } => {
                // let aspect_ratio = aspect_ratio.unwrap_or(framebuffer_size.x / framebuffer_size.y);
                let aspect_ratio = framebuffer_size.x / framebuffer_size.y;
                mat4::perspective(y_fov * aspect_ratio, aspect_ratio, z_near, z_far)
            }
        }
    }
}

impl<'a> From<gltf::camera::Projection<'a>> for Projection {
    fn from(value: gltf::camera::Projection) -> Self {
        match value {
            gltf::camera::Projection::Orthographic(orthographic) => Self::Orthographic {
                x_mag: orthographic.xmag(),
                y_mag: orthographic.ymag(),
                z_near: orthographic.znear(),
                z_far: orthographic.zfar(),
            },
            gltf::camera::Projection::Perspective(perspective) => Self::Perspective {
                aspect_ratio: perspective.aspect_ratio(),
                y_fov: perspective.yfov(),
                z_near: perspective.znear(),
                z_far: perspective.zfar().unwrap_or(1000.0),
            },
        }
    }
}

/// FreeCamera object can be used in games.
/// It's a wrapper around [Camera], that helps control/move it without scene/node tree.
pub struct FreeCamera {
    pub projection: Projection,
    pub pos: vec3<f32>,
    pub rot_h: Angle<f32>,
    pub rot_v: Angle<f32>,
}

impl geng::AbstractCamera3d for FreeCamera {
    fn view_matrix(&self) -> mat4<f32> {
        // mat4::translate(vec3(0.0, 0.0, -self.distance)) *
        mat4::rotate_x(-self.rot_v) * mat4::rotate_z(-self.rot_h) * mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        self.projection.matrix(framebuffer_size)
    }
}
