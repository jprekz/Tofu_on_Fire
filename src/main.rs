use amethyst::{
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::*,
    ui::{DrawUi, UiBundle},
    utils::{application_root_dir, fps_counter::FPSCounterBundle},
};

mod ai;
mod audio;
mod bundle;
mod common;
mod components;
mod game;
mod prefab;
mod respawn;
mod systems;
mod weapon;

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default())
        .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
        .level_for("amethyst_assets", amethyst::LogLevelFilter::Info)
        .start();

    let app_root = application_root_dir();

    let render_bundle = {
        let path = format!("{}/resources/display_config.ron", app_root);
        let config = DisplayConfig::load(&path);
        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                .with_pass(DrawFlat2D::new().with_transparency(
                    ColorMask::all(),
                    ALPHA,
                    Some(DepthMode::LessEqualWrite),
                ))
                .with_pass(DrawUi::new()),
        );
        RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&[])
    };

    let input_bundle = {
        let path = format!("{}/resources/bindings_config.ron", app_root);
        InputBundle::<String, String>::new().with_bindings_from_file(path)?
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(AudioBundle::new(|music: &mut audio::Music| {
            music.music.next()
        }))?
        .with_bundle(FPSCounterBundle)?
        .with_bundle(input_bundle)?
        .with_bundle(bundle::GameBundle::default())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(render_bundle)?
        .with_bundle(UiBundle::<String, String>::new())?;

    let mut game = Application::new("./", game::Game::default(), game_data)?;

    game.run();

    Ok(())
}
