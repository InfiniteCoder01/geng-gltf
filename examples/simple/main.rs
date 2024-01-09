use geng::prelude::*;

pub struct Camera {
    pub fov: f32,
    pub pos: vec3<f32>,
    pub distance: f32,
    pub rot_h: Angle<f32>,
    pub rot_v: Angle<f32>,
}

impl Camera {
    pub fn eye_pos(&self) -> vec3<f32> {
        let v = vec2(self.distance, 0.0).rotate(self.rot_v);
        self.pos + vec3(0.0, -v.y, v.x)
    }
}

impl geng::AbstractCamera3d for Camera {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::translate(vec3(0.0, 0.0, -self.distance))
            * mat4::rotate_x(-self.rot_v)
            * mat4::rotate_z(-self.rot_h)
            * mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        mat4::perspective(
            self.fov,
            framebuffer_size.x / framebuffer_size.y,
            0.1,
            1000.0,
        )
    }
}
fn main() {
    logger::init();
    geng::setup_panic_handler();
    Geng::run("Hello, World!", |geng| async move {
        let mut events = geng.window().events();

        let model =
            geng_gltf::Model::load(geng.ugli(), "/home/infinitecoder/Downloads/Test.glb").unwrap();

        let program = geng
            .asset_manager()
            .load::<ugli::Program>("examples/simple/shader.glsl")
            .await
            .unwrap();

        let mut camera = Camera {
            fov: f32::PI / 3.0,
            pos: vec3(0.0, 0.0, 1.0),
            distance: 5.0,
            rot_h: Angle::ZERO,
            rot_v: Angle::from_radians(f32::PI / 3.0),
        };

        while let Some(event) = events.next().await {
            match event {
                geng::Event::MousePress { .. } => {
                    geng.window().lock_cursor();
                }
                geng::Event::MouseRelease { .. } => {
                    geng.window().unlock_cursor();
                }
                geng::Event::RawMouseMove { delta, .. } => {
                    let sense = 0.01;
                    camera.rot_h += Angle::from_radians(delta.x as f32 * sense);
                    camera.rot_v = (camera.rot_v + Angle::from_radians(delta.y as f32 * sense))
                        .clamp_range(Angle::ZERO..=Angle::from_radians(f32::PI));
                }

                geng::Event::Draw => {
                    geng.window().with_framebuffer(|framebuffer| {
                        let framebuffer_size = framebuffer.size().map(|x| x as f32);
                        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

                        model.draw(
                            framebuffer,
                            &program,
                            (
                                ugli::uniforms! {
                                    u_model_matrix: mat4::identity(),
                                    u_joint_mat: [
                                        mat4::identity(),
                                        mat4::identity(),
                                    ],
                                    u_light_pos: vec3(1.2, 1.0, 2.0),
                                },
                                camera.uniforms(framebuffer_size),
                            ),
                            ugli::DrawParameters {
                                depth_func: Some(ugli::DepthFunc::Less),
                                ..default()
                            },
                        )
                    });
                }
                _ => {}
            }
        }
    });
}
