use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, RonFormat},
    core::nalgebra::*,
    core::transform::Transform,
    input::is_key_down,
    prelude::*,
    renderer::*,
    winit::VirtualKeyCode,
    utils::fps_counter::FPSCounter,
};

use crate::prefab::*;
use crate::weapon::*;

pub struct Game;

impl SimpleState for Game {
    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        println!("{}", data.world.read_resource::<FPSCounter>().frame_fps());
        Trans::None
    }

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let sprite_sheet_handle = load_sprite_sheet(world);

        initialise_camera(world);

        world.add_resource(sprite_sheet_handle);

        let weapon_list = WeaponList::load("resources/weapon_list.ron");
        world.add_resource(weapon_list);

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/prefab.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MapTilePrefab>| {
            loader.load("resources/map1.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();
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

fn initialise_camera(world: &mut World) {
    use amethyst::renderer::{Camera, Projection};

    let mut transform = Transform::default();
    transform.set_position(Vector3::new(0.0, 0.0, 100.0));

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 640.0, 480.0, 0.0, // Y axis reversed
        )))
        .with(transform)
        .build();
}
