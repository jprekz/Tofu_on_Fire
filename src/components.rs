use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    derive::PrefabData,
    ecs::prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Player {
    pub speed: f32,
    #[serde(default = "zero")]
    pub trigger_timer: u32,
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Enemy {
    pub speed: f32,
    #[serde(default = "zero")]
    pub trigger_timer: u32,
}

#[derive(Component, Clone, Debug)]
pub struct Wall;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct Bullet {
    pub timer_limit: u32,
    pub timer_count: u32,
    pub reflect_limit: u32,
    pub reflect_count: u32,
}
impl Bullet {
    pub fn new(timer_limit: u32, reflect_limit: u32) -> Bullet {
        Bullet {
            timer_limit,
            timer_count: 0,
            reflect_limit,
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
}
impl Default for Rigidbody {
    fn default() -> Rigidbody {
        Rigidbody {
            velocity: Vector2::zeros(),
            acceleration: Vector2::zeros(),
            drag: 0.0,
            bounciness: 0.0,
        }
    }
}

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct RectCollider<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub width: f32,
    pub height: f32,
    #[serde(default = "Vector2::zeros")]
    pub collision: Vector2<f32>,
    #[serde(default)]
    phantom: std::marker::PhantomData<T>,
}
impl<T> RectCollider<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(width: f32, height: f32) -> RectCollider<T> {
        RectCollider {
            width: width,
            height: height,
            collision: Vector2::zeros(),
            phantom: std::marker::PhantomData,
        }
    }
}
