use amethyst::{
    assets::{AssetStorage, Handle, Loader, Prefab, PrefabLoader, RonFormat},
    core::nalgebra::*,
    core::Time,
    core::Transform,
    ecs::prelude::*,
    input::is_key_down,
    input::InputHandler,
    prelude::*,
    renderer::*,
    ui::*,
    utils::fps_counter::FPSCounter,
    winit::VirtualKeyCode,
};

use crate::audio::*;
use crate::components::*;
use crate::prefab::*;
use crate::weapon::*;

#[derive(Default)]
pub struct Game {
    fps_display: Option<Entity>,
    player_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    ai_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    enemy_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    player_weapon: usize,
    ai_weapon: usize,
    enemy_weapon: usize,
}

impl SimpleState for Game {
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::Escape) => Trans::Quit,
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::F1) => {
                let StateData { world, .. } = data;
                if let Err(e) = MapPrefabData::save(world) {
                    log::warn!("Failed to save map: {}", e);
                }
                Trans::None
            }
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::F2) => {
                let StateData { world, .. } = data;
                MapPrefabData::reload(world);
                Trans::None
            }
            _ => Trans::None,
        }
    }

    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        // display fps
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
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        // spawn npc
        if world.read_resource::<Time>().frame_number() % 128 == 0 {
            let (mut army_count, mut enemy_count) = (0u32, 0u32);
            for player in world.read_storage::<Player>().join() {
                if player.team == 0 {
                    army_count += 1;
                } else {
                    enemy_count += 1;
                }
            }

            if army_count < 10 {
                if let Some(point) = get_spawn_point(world, 0) {
                    let mut transform = Transform::default();
                    transform.set_xyz(point.x, point.y, 0.0);
                    world
                        .create_entity()
                        .with(
                            self.ai_prefab_handle
                                .clone()
                                .expect("Failed to get prefab handle??"),
                        )
                        .with(transform)
                        .with(Player {
                            weapon: self.ai_weapon,
                            ..Default::default()
                        })
                        .build();
                    self.ai_weapon = (self.ai_weapon + 1) % 3;
                }
            }
            if enemy_count < 10 {
                if let Some(point) = get_spawn_point(world, 1) {
                    let mut transform = Transform::default();
                    transform.set_xyz(point.x, point.y, 0.0);
                    world
                        .create_entity()
                        .with(
                            self.enemy_prefab_handle
                                .clone()
                                .expect("Failed to get prefab handle??"),
                        )
                        .with(transform)
                        .with(Player {
                            team: 1,
                            weapon: self.enemy_weapon,
                            ..Default::default()
                        })
                        .build();
                    self.enemy_weapon = (self.enemy_weapon + 1) % 3;
                }
            }
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        let pressed_any_key = {
            let input = world.read_resource::<InputHandler<String, String>>();

            let shot = input.action_is_down("shot").unwrap_or(false);
            let change = input.action_is_down("change").unwrap_or(false);
            shot || change
        };

        if pressed_any_key {
            // hide title
            world.exec(
                |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                    if let Some(entity) = finder.find("title") {
                        if hidden.insert(entity, HiddenPropagate).is_err() {
                            log::warn!("Failed to insert HiddenPropagate component");
                        }
                    }
                },
            );

            // spawn player
            if world.read_storage::<Playable>().join().next().is_none() {
                if let Some(point) = get_spawn_point(world, 0) {
                    let mut transform = Transform::default();
                    transform.set_xyz(point.x, point.y, 0.0);
                    world
                        .create_entity()
                        .with(
                            self.player_prefab_handle
                                .clone()
                                .expect("Failed to get prefab handle??"),
                        )
                        .with(transform)
                        .with(Player {
                            weapon: self.player_weapon,
                            ..Default::default()
                        })
                        .build();
                    self.player_weapon = (self.player_weapon + 1) % 3;
                }
            }

            return Trans::Push(Box::new(Playing::default()));
        }

        Trans::None
    }

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let sprite_sheet_handle = load_sprite_sheet(world);
        world.add_resource(sprite_sheet_handle);

        let weapon_list = WeaponList::load("resources/weapon_list.ron");
        world.add_resource(weapon_list);

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MapPrefabData>| {
            loader.load("resources/map.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/camera.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/player.ron", RonFormat, (), ())
        });
        self.player_prefab_handle = Some(prefab_handle);

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/ai.ron", RonFormat, (), ())
        });
        self.ai_prefab_handle = Some(prefab_handle);

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/enemy.ron", RonFormat, (), ())
        });
        self.enemy_prefab_handle = Some(prefab_handle);

        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("resources/ui.ron", ());
        });

        initialise_audio(world);
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        // show title
        world.exec(
            |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                if let Some(entity) = finder.find("title") {
                    if hidden.remove(entity).is_none() {
                        log::warn!("Failed to remove HiddenPropagate component");
                    }
                }
            },
        );
    }
}

#[derive(Default)]
pub struct Playing;

impl SimpleState for Playing {
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::Escape) => Trans::Quit,
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        if world.read_storage::<Playable>().join().next().is_none() {
            return Trans::Pop;
        }

        Trans::None
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

fn get_spawn_point(world: &mut World, team: u32) -> Option<Vector2<f32>> {
    world.exec(
        |(spawnpoints, transforms): (ReadStorage<'_, SpawnPoint>, WriteStorage<'_, Transform>)| {
            (&spawnpoints, &transforms)
                .join()
                .filter(|(spawnpoint, _)| spawnpoint.team == team)
                .next()
                .map(|(_, transform)| transform.translation().xy())
        },
    )
}
