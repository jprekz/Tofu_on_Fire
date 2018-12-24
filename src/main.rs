use amethyst::{
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{ColorMask, DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage, ALPHA},
    utils::application_root_dir,
};

mod bundle;
mod collision;
mod components;
mod game;
mod prefab;
mod systems;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let render_bundle = {
        let path = format!("{}/resources/display_config.ron", app_root);
        let config = DisplayConfig::load(&path);
        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
                .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None)),
        );
        RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor()
    };

    let input_bundle = {
        let path = format!("{}/resources/bindings_config.ron", app_root);
        InputBundle::<String, String>::new().with_bindings_from_file(path)?
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(render_bundle)?
        .with_bundle(input_bundle)?
        .with_bundle(TransformBundle::new())?
        .with_bundle(bundle::GameBundle::default())?;

    let mut game = Application::new("./", game::Game, game_data)?;

    game.run();

    Ok(())
}
