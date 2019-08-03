use amethyst::{
    assets::{AssetStorage, Loader, PrefabLoader, RonFormat},
    core::transform::*,
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
use crate::resource::*;
use crate::state::*;

use crate::common::pause::Pause;


#[derive(Default)]
pub struct Game {
    fps_display: Option<Entity>,
    released: bool,
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

        if pressed_any_key && self.released {
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

        if !pressed_any_key {
            self.released = true;
        }

        Trans::None
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        self.released = false;

        world.write_resource::<Pause>().off();

        macro_rules! skip_fail {
            ($res:expr) => {
                match $res {
                    Ok(val) => val,
                    Err(e) => {
                        log::warn!("{} (L{})", e, line!());
                        continue;
                    }
                }
            };
        }

        // delete entities
        world.exec(
            |(entities, players, hierarchy, bullets, items, particles): (
                Entities,
                ReadStorage<'_, Player>,
                WriteExpect<'_, ParentHierarchy>,
                ReadStorage<'_, Bullet>,
                ReadStorage<'_, Item>,
                ReadStorage<'_, Particle>,
            )| {
                for (entity, _) in (&entities, &players).join() {
                    skip_fail!(entities.delete(entity));
                    for entity in hierarchy.all_children_iter(entity) {
                        skip_fail!(entities.delete(entity));
                    }
                }
                for (entity, _) in (&entities, &bullets).join() {
                    skip_fail!(entities.delete(entity));
                }
                for (entity, _) in (&entities, &items).join() {
                    skip_fail!(entities.delete(entity));
                }
                for (entity, _) in (&entities, &particles).join() {
                    skip_fail!(entities.delete(entity));
                }
            },
        );

        // reset area
        world.exec(
            |(areas, mut transforms): (ReadStorage<'_, Area>, WriteStorage<'_, Transform>)| {
                for (_, transform) in (&areas, &mut transforms).join() {
                    transform.set_x(352.0);
                }
            },
        );

        // reset score
        world.add_resource(Score { score: vec![0, 0] });

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

        #[cfg(feature = "include_resources")]
        let weapon_list =
            WeaponList::load_bytes(include_bytes!("../../resources/weapon_list.ron")).unwrap();
        #[cfg(not(feature = "include_resources"))]
        let weapon_list = WeaponList::load("resources/weapon_list.ron");
        world.add_resource(weapon_list);

        world.add_resource(Score { score: vec![0, 0] });

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MapPrefabData>| {
            #[cfg(feature = "include_resources")]
            return loader.load_from_data(
                Config::load_bytes(include_bytes!("../../resources/map.ron")).unwrap(),
                (),
            );
            #[cfg(not(feature = "include_resources"))]
            return loader.load("resources/map.ron", RonFormat, (), ());
        });
        world.create_entity().with(prefab_handle).build();

        let prefab_handle = world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            #[cfg(feature = "include_resources")]
            return loader.load_from_data(
                Config::load_bytes(include_bytes!("../../resources/camera.ron")).unwrap(),
                (),
            );
            #[cfg(not(feature = "include_resources"))]
            return loader.load("resources/camera.ron", RonFormat, (), ());
        });
        world.create_entity().with(prefab_handle).build();


        #[cfg(feature = "include_resources")]
        let ui_handle = world.exec(
            |(loader, storage): (ReadExpect<'_, Loader>, Read<'_, AssetStorage<UiPrefab>>)| {
                use amethyst::assets::SimpleFormat;
                loader.load_from_data(
                    UiFormat::<NoCustomUi>::default()
                        .import(include_bytes!("../../resources/ui.ron").to_vec(), ())
                        .unwrap(),
                    (),
                    &storage,
                )
            },
        );
        #[cfg(not(feature = "include_resources"))]
        let ui_handle = world.exec(|loader: UiLoader<'_>| loader.load("resources/ui.ron", ()));
        world.create_entity().with(ui_handle).build();

        let respawn_handler = RespawnHandler::initialize(world);
        world.add_resource(respawn_handler);

        initialise_audio(world);
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
