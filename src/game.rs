use amethyst::{
    assets::{AssetStorage, Loader},
    core::cgmath::*,
    core::transform::Transform,
    input::is_key_down,
    prelude::*,
    renderer::{
        MaterialTextureSet, PngFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata,
    },
    winit::VirtualKeyCode,
};
use components::*;
use config::*;

pub struct Game;

impl<'a, 'b> SimpleState<'a, 'b> for Game {
    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans<'a, 'b> {
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

    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let sprite_sheet_handle = load_sprite_sheet(world);

        initialise_camera(world);
        initialise_player(world, sprite_sheet_handle.clone());
        initialise_map(world, sprite_sheet_handle.clone());

        world.add_resource(sprite_sheet_handle);
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
    let texture_id = 0;
    let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
    material_texture_set.insert(texture_id, texture_handle);
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/spritesheet.ron",
        SpriteSheetFormat,
        texture_id,
        (),
        &sprite_sheet_store,
    )
}

fn initialise_camera(world: &mut World) {
    use amethyst::renderer::{Camera, Projection};

    let transform = Transform {
        translation: Vector3::new(0.0, 0.0, 1.0),
        ..Default::default()
    };

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 640.0, 0.0, 480.0,
        ))).with(transform)
        .build();
}

fn initialise_player(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let spawn1 = {
        let config = &world.read_resource::<MapConfig>();
        config.spawn1
    };

    let transform = Transform {
        translation: Vector3::new(spawn1.0, spawn1.1, 0.0),
        ..Default::default()
    };

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
        flip_horizontal: false,
        flip_vertical: false,
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Player {
            speed: 2.0,
            trigger_timer: 0,
        }).with(RectCollider::<Player>::new(16.0, 16.0))
        .with(transform)
        .with(Rigidbody {
            drag: 0.5,
            ..Default::default()
        }).build();
}

fn initialise_map(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let (size, map) = {
        let config = world.read_resource::<MapConfig>();
        (config.size, config.map.clone())
    };

    let mut new_wall = |width: f32, height: f32, x: f32, y: f32| {
        let transform = Transform {
            translation: Vector3::new(x, y, 0.0),
            scale: Vector3::new(width / 32.0, height / 32.0, 1.0),
            ..Default::default()
        };

        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: 2,
            flip_horizontal: false,
            flip_vertical: false,
        };

        world
            .create_entity()
            .with(sprite_render)
            .with(RectCollider::<Wall>::new(width, height))
            .with(transform)
            .build();
    };

    for map_y in 0..size.1 {
        for map_x in 0..size.0 {
            let tile = map[(map_y * size.0 + map_x) as usize];
            if tile > 0 {
                let width = tile as f32 * 32.0;
                let height = 32.0;
                let x = (map_x * 32) as f32 + width / 2.0;
                let y = (map_y * 32) as f32 + height / 2.0;
                new_wall(width, height, x, y);
            }
            if tile < 0 {
                let width = 32.0;
                let height = -tile as f32 * 32.0;
                let x = (map_x * 32) as f32 + width / 2.0;
                let y = (map_y * 32) as f32 + height / 2.0;
                new_wall(width, height, x, y);
            }
        }
    }
}
