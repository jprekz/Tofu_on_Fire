use amethyst::{
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::*,
    ui::UiBundle,
    utils::{application_root_dir, fps_counter::FPSCounterBundle},
    window::WindowBundle,
};

mod ai;
mod audio;
mod bundle;
mod common;
mod components;
mod game;
mod prefab;
mod systems;
mod weapon;

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default())
        .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
        .level_for("amethyst_assets", amethyst::LogLevelFilter::Info)
        .start();

    let app_root = application_root_dir()?;

    let window_bundle = {
        let path = app_root.join("resources/display_config.ron");
        WindowBundle::from_config_path(path)
    };

    let input_bundle = {
        let path = app_root.join("resources/bindings_config.ron");
        InputBundle::<StringBindings>::new().with_bindings_from_file(path)?
    };

    let game_data = GameDataBuilder::default()
        .with_bundle(AudioBundle::default())?
        .with_bundle(FPSCounterBundle)?
        .with_bundle(input_bundle)?
        .with_bundle(bundle::GameBundle::default())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(window_bundle)?
        .with_bundle(UiBundle::<DefaultBackend, StringBindings>::new())?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderGraph::default(),
        ));

    let mut game = Application::new("./", game::Game::default(), game_data)?;

    game.run();

    Ok(())
}


use amethyst::{
    ecs::{ReadExpect, Resources, SystemData},
    renderer::{
        pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc},
        rendy::{
            factory::Factory,
            graph::{
                present::PresentNode,
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{
                command::{ClearDepthStencil, ClearValue},
                format::Format,
                image::Kind,
            },
        },
        types::DefaultBackend,
        GraphCreator,
    },
    ui::DrawUiDesc,
    window::{ScreenDimensions, Window},
};

#[derive(Default)]
pub struct RenderGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for RenderGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }
        return self.dirty;
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        self.dirty = false;

        let window = <ReadExpect<'_, Window>>::fetch(res);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);
        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            // clear screen to black
            Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let sprite_pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        let ui_pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_dependency(sprite_pass)
                .with_group(DrawUiDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder.add_node(
            PresentNode::builder(factory, surface, color)
                .with_dependency(sprite_pass)
                .with_dependency(ui_pass),
        );

        graph_builder
    }
}
