use amethyst::{assets::*, core::nalgebra::*, core::Transform, ecs::prelude::*};

#[cfg(feature = "include_resources")]
use amethyst::prelude::Config;

use crate::components::*;
use crate::prefab::*;

#[derive(Clone)]
pub struct RespawnHandler {
    player_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    ai_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    enemy_prefab_handle: Option<Handle<Prefab<MyPrefabData>>>,
    ai_weapon: usize,
    enemy_weapon: usize,
}

impl RespawnHandler {
    pub fn initialize(world: &mut World) -> Self {
        RespawnHandler {
            player_prefab_handle: world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
                #[cfg(feature = "include_resources")]
                return Some(loader.load_from_data(
                    Config::load_bytes(include_bytes!("../resources/player.ron")).unwrap(),
                    (),
                ));
                #[cfg(not(feature = "include_resources"))]
                return Some(loader.load("resources/player.ron", RonFormat, (), ()));
            }),
            ai_prefab_handle: world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
                #[cfg(feature = "include_resources")]
                return Some(loader.load_from_data(
                    Config::load_bytes(include_bytes!("../resources/ai.ron")).unwrap(),
                    (),
                ));
                #[cfg(not(feature = "include_resources"))]
                return Some(loader.load("resources/ai.ron", RonFormat, (), ()));
            }),
            enemy_prefab_handle: world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
                #[cfg(feature = "include_resources")]
                return Some(loader.load_from_data(
                    Config::load_bytes(include_bytes!("../resources/enemy.ron")).unwrap(),
                    (),
                ));
                #[cfg(not(feature = "include_resources"))]
                return Some(loader.load("resources/enemy.ron", RonFormat, (), ()));
            }),
            ai_weapon: 0,
            enemy_weapon: 0,
        }
    }

    pub fn respawn_npc(&mut self, world: &mut World) {
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

    pub fn respawn_player(&mut self, world: &mut World, weapon: usize) {
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
                        weapon: weapon,
                        ..Default::default()
                    })
                    .build();
            }
        }
    }
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
