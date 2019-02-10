use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    derive::PrefabData,
    ecs::prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

pub use crate::common::collision2d::{ColliderResult, RectCollider, Rigidbody};

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Player {
    pub team: u32,
    pub weapon: usize,
    pub hp: f32,

    #[serde(skip, default = "Vector2::zeros")]
    pub input_move: Vector2<f32>,
    #[serde(skip, default = "Vector2::zeros")]
    pub input_aim: Vector2<f32>,
    #[serde(skip)]
    pub input_shot: bool,
    #[serde(skip)]
    pub input_change: bool,
    #[serde(skip, default = "zero")]
    pub trigger_timer: u32,
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Playable {
    #[serde(skip)]
    pub input_change_hold: bool,
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct AI {
    #[serde(skip)]
    pub state: AIState,
}
#[derive(Clone, Debug)]
pub enum AIState {
    Go(Entity),
    Away(Entity),
    Neutral,
}
impl Default for AIState {
    fn default() -> AIState {
        AIState::Neutral
    }
}

#[derive(Component, PrefabData, Deserialize, Serialize, Default, Clone, Debug)]
#[storage(NullStorage)]
#[prefab(Component)]
pub struct Reticle;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct SpawnPoint {
    pub team: u32,
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Bullet {
    pub team: u32,
    pub timer_limit: u32,
    pub reflect_limit: u32,
    pub knockback: f32,
    pub slowing: f32,
    pub pierce: bool,

    #[serde(skip, default = "zero")]
    pub timer_count: u32,
    #[serde(skip, default = "zero")]
    pub reflect_count: u32,
}
impl Bullet {
    pub fn new(
        team: u32,
        timer_limit: u32,
        reflect_limit: u32,
        knockback: f32,
        slowing: f32,
        pierce: bool,
    ) -> Bullet {
        Bullet {
            team,
            timer_limit,
            reflect_limit,
            knockback,
            slowing,
            pierce,
            timer_count: 0,
            reflect_count: 0,
        }
    }
}
