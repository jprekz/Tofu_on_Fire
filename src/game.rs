use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, RonFormat},
    core::Time,
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
use crate::respawn::*;
use crate::weapon::*;

pub struct Score {
    pub score: Vec<u32>,
}

#[derive(Default)]
pub struct Game {
    fps_display: Option<Entity>,
}

impl SimpleState for Game {
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

    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
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
    }

    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        // spawn npc
        if world.read_resource::<Time>().frame_number() % 128 == 0 {
            let mut rh = world.read_resource::<RespawnHandler>().clone();
            rh.respawn_npc(world);
            *world.write_resource::<RespawnHandler>() = rh;
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        let pressed_any_key = {
            let input = world.read_resource::<InputHandler<String, String>>();

            let shot = input.action_is_down("shot").unwrap_or(false);
            let hold = input.action_is_down("hold").unwrap_or(false);
            shot || hold
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

            return Trans::Push(Box::new(Select::default()));
        }

        Trans::None
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

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let sprite_sheet_handle = load_sprite_sheet(world);
        world.add_resource(sprite_sheet_handle);

        let weapon_list = WeaponList::load("resources/weapon_list.ron");
        world.add_resource(weapon_list);

        world.add_resource(Score { score: vec![0, 0] });

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MapPrefabData>| {
            loader.load("resources/map.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("resources/camera.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();


        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("resources/ui.ron", ());
        });

        let respawn_handler = RespawnHandler::initialize(world);
        world.add_resource(respawn_handler);

        initialise_audio(world);
    }
}

#[derive(Default)]
pub struct Select {
    selecting: u32,
    released: bool,
    timer: i32,
}

impl SimpleState for Select {
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

        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer < 0 {
            self.timer += 1;
        }

        let (up, down, shot, mouse_vec) = {
            let input = world.read_resource::<InputHandler<String, String>>();

            let move_y = input.axis_value("move_y").unwrap_or(0.0);
            let dpad_y = input.axis_value("dpad_y").unwrap_or(0.0);
            let left_y = input.axis_value("left_y").unwrap_or(0.0);
            let y = move_y + dpad_y + left_y;
            let up = y < -0.1;
            let down = y > 0.1;
            let shot = input.action_is_down("shot").unwrap_or(false);
            let mouse_vec = input.mouse_position();
            (up, down, shot, mouse_vec)
        };

        if up && self.timer >= 0 {
            self.selecting = (self.selecting + 2) % 3;
            self.timer = -10;
        }
        if down && self.timer <= 0 {
            self.selecting = (self.selecting + 1) % 3;
            self.timer = 10;
        }
        if let Some((x, y)) = mouse_vec {
            if x >= 50.0 && x < 520.0 {
                if y >= 510.0 && y < 620.0 {
                    self.selecting = 0;
                }
                if y >= 630.0 && y < 740.0 {
                    self.selecting = 1;
                }
                if y >= 750.0 && y < 860.0 {
                    self.selecting = 2;
                }
            }
        }

        world.exec(
            |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                if let Some(entity) = finder.find("menu1") {
                    if self.selecting == 0 {
                        hidden.remove(entity);
                    } else {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                }
                if let Some(entity) = finder.find("menu2") {
                    if self.selecting == 1 {
                        hidden.remove(entity);
                    } else {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                }
                if let Some(entity) = finder.find("menu3") {
                    if self.selecting == 2 {
                        hidden.remove(entity);
                    } else {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                }
            },
        );

        if shot && self.released {
            // spawn player
            let mut rh = world.read_resource::<RespawnHandler>().clone();
            rh.respawn_player(world, self.selecting as usize);
            *world.write_resource::<RespawnHandler>() = rh;

            // hide menu
            world.exec(
                |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                    if let Some(entity) = finder.find("menu1") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                    if let Some(entity) = finder.find("menu2") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                    if let Some(entity) = finder.find("menu3") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                },
            );

            return Trans::Push(Box::new(Playing::default()));
        }
        if !shot {
            self.released = true;
        }

        Trans::None
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.released = false;
    }
}

#[derive(Default)]
pub struct Playing {
    timer: Option<i32>,
}

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

        if self.timer.is_none() && world.read_storage::<Playable>().join().next().is_none() {
            self.timer = Some(60);
        }

        if let Some(ref mut timer) = self.timer {
            *timer -= 1;
            if *timer < 0 {
                return Trans::Pop;
            }
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
