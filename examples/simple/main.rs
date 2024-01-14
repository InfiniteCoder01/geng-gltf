use geng::prelude::*;

fn main() {
    logger::init();
    geng::setup_panic_handler();

    Geng::run_with(
        &geng::ContextOptions {
            window: geng::window::Options::new("glTF"),
            shader_lib: hashmap! {
                "gltf".to_owned() => geng_gltf::prelude_shader(),
                "gltf-pbr".to_owned() => geng_gltf::pbr_shader(),
            },
            ..default()
        },
        |geng| async move {
            let mut model =
                geng_gltf::Model::load(geng.ugli(), "examples/simple/simple.glb").unwrap();

            let program = geng
                .asset_manager()
                .load::<ugli::Program>("examples/simple/shader.glsl")
                .await
                .unwrap();

            let start = std::time::Instant::now();

            let mut events = geng.window().events();
            while let Some(event) = events.next().await {
                if event == geng::Event::Draw {
                    geng.window().with_framebuffer(|framebuffer| {
                        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

                        model.draw(
                            framebuffer,
                            &program,
                            Some("Camera"),
                            ugli::uniforms! {
                                u_light_pos: vec3(1.2, 1.0, 2.0),
                            },
                            ugli::DrawParameters {
                                depth_func: Some(ugli::DepthFunc::Less),
                                ..default()
                            },
                        )
                    });
                    if let Err(err) = geng.ugli().try_check() {
                        log::error!("Ugli Error: {:?}", err);
                    }
                }
            }
        },
    );
}
