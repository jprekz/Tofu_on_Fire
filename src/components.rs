use amethyst::{
    core::cgmath::*,
    ecs::prelude::*,
};

#[derive(Component, Debug)]
pub struct Player {
    pub speed: f32,
    pub trigger_timer: u32,
}

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct Bullet;

#[derive(Component, Debug)]
pub struct Rigidbody {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub drag: f32,
    pub bounciness: f32,
}
impl Default for Rigidbody {
    fn default() -> Rigidbody {
        Rigidbody {
            velocity: Vector2::zero(),
            acceleration: Vector2::zero(),
            drag: 0.0,
            bounciness: 0.0,
        }
    }
}

use std::marker::PhantomData;
#[derive(Component, Debug)]
pub struct RectCollider<T>
where
    T: Send + Sync + 'static,
{
    pub width: f32,
    pub height: f32,
    pub collision: Vector2<f32>,
    phantom: PhantomData<T>,
}
impl<T> RectCollider<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(width: f32, height: f32) -> RectCollider<T> {
        RectCollider {
            width: width,
            height: height,
            collision: Vector2::<f32>::zero(),
            phantom: PhantomData,
        }
    }
}
