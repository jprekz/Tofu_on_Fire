use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    derive::PrefabData,
    ecs::prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

pub use crate::collision::RectCollider;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Player {
    pub team: u32,
    pub speed: f32,

    #[serde(skip, default = "Vector2::zeros")]
    pub input_move: Vector2<f32>,
    #[serde(skip, default = "Vector2::zeros")]
    pub input_aim: Vector2<f32>,
    #[serde(skip, default)]
    pub input_shot: bool,
    #[serde(skip, default = "zero")]
    pub trigger_timer: u32,
    #[serde(skip, default = "zero")]
    pub damage: u32,
    #[serde(skip, default = "Vector2::zeros")]
    pub knock_back: Vector2<f32>,
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Playable;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct AI;

#[derive(Component, Clone, Debug)]
pub struct Wall;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Bullet {
    pub team: u32,
    pub timer_limit: u32,
    pub reflect_limit: u32,

    #[serde(skip, default = "zero")]
    pub timer_count: u32,
    #[serde(skip, default = "zero")]
    pub reflect_count: u32,
}
impl Bullet {
    pub fn new(team: u32, timer_limit: u32, reflect_limit: u32) -> Bullet {
        Bullet {
            team,
            timer_limit,
            reflect_limit,
            timer_count: 0,
            reflect_count: 0,
        }
    }
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
#[serde(default)]
pub struct Rigidbody {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub drag: f32,
    pub bounciness: f32,
    pub auto_rotate: bool,
}
impl Default for Rigidbody {
    fn default() -> Rigidbody {
        Rigidbody {
            velocity: Vector2::zeros(),
            acceleration: Vector2::zeros(),
            drag: 0.0,
            bounciness: 0.0,
            auto_rotate: false,
        }
    }
}
