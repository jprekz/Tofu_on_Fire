#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use amethyst::{
    audio::{AudioBundle, DjSystemDesc},
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
    window::DisplayConfig,
};

mod ai;
mod audio;
mod bundle;
mod common;
mod components;
mod prefab;
mod resources;
mod state;
mod systems;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let render_bundle = {
        #[cfg(not(feature = "include_resources"))]
        let mut config = DisplayConfig::load(app_root.join("resources/display_config.ron"))?;

        #[cfg(feature = "include_resources")]
        let mut config =
            DisplayConfig::load_bytes(include_bytes!("../resources/display_config.ron"))?;

        let default_dimensions = (1280, 960);
        let dimensions = Some(
            std::fs::File::open("./config.txt")
                .map(|f| ron::de::from_reader(f).unwrap_or(default_dimensions))
                .unwrap_or(default_dimensions),
        );
        config.min_dimensions = dimensions;
        config.max_dimensions = dimensions;

        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(RenderToWindow::from_config(config).with_clear([0.0, 0.0, 0.0, 1.0]))
            .with_plugin(RenderFlat2D::default())
            .with_plugin(RenderUi::default())
    };

    let input_bundle = {
        #[cfg(not(feature = "include_resources"))]
        let bundle = InputBundle::<StringBindings>::new()
            .with_bindings_from_file(app_root.join("resources/bindings_config.ron"))?;

        #[cfg(feature = "include_resources")]
        let bundle = InputBundle::<StringBindings>::new().with_bindings(Config::load_bytes(
            include_bytes!("../resources/bindings_config.ron"),
        )?);

        bundle
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(AudioBundle::default())?
        .with_system_desc(
            DjSystemDesc::new(|music: &mut audio::Music| music.music.next()),
            "dj_system",
            &[],
        )
        .with_bundle(FpsCounterBundle)?
        .with_bundle(input_bundle)?
        .with_bundle(bundle::GameBundle::default())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(render_bundle)?
        .with_bundle(UiBundle::<StringBindings>::new())?;

    let mut game = Application::new("./", state::Game::default(), game_data)?;

    game.run();

    Ok(())
}
