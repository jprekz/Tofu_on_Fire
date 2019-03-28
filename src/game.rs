use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, RonFormat},
    core::Time,
    ecs::prelude::Entity,
    input::is_key_down,
    prelude::*,
    renderer::*,
    ui::*,
    utils::fps_counter::FPSCounter,
    winit::VirtualKeyCode,
};

use crate::prefab::*;
use crate::weapon::*;

#[derive(Default)]
pub struct Game {
    fps_display: Option<Entity>,
}

impl SimpleState for Game {
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::Escape) => Trans::Quit,
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::S) => {
                let StateData { world, .. } = data;
                MapPrefabData::save(world);
                Trans::None
            }
            _ => Trans::None,
        }
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("fps_text") {
                    self.fps_display = Some(entity);
                }
            });
        }
        let mut ui_text = world.write_storage::<UiText>();
        if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FPSCounter>().sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }

        Trans::None
    }

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let sprite_sheet_handle = load_sprite_sheet(world);
        world.add_resource(sprite_sheet_handle);

        let weapon_list = WeaponList::load("resources/weapon_list.ron");
        world.add_resource(weapon_list);

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MapPrefabData>| {
            loader.load("resources/map.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/player.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/ai.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/enemy.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle.clone()).build();
        world.create_entity().with(prefab_handle).build();

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("resources/fps.ron", ());
        });
    }
}

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/spritesheet.ron",
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}
