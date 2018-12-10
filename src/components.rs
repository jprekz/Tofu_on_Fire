use amethyst::{core::nalgebra::*, ecs::prelude::*};
use specs_derive::Component;

#[derive(Component, Debug)]
pub struct Player {
    pub speed: f32,
    pub trigger_timer: u32,
}

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
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
            velocity: Vector2::zeros(),
            acceleration: Vector2::zeros(),
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
            collision: Vector2::zeros(),
            phantom: PhantomData,
        }
    }
}
