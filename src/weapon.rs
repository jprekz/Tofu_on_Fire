use serde_derive::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Deserialize, Serialize, Default)]
pub struct WeaponList {
    pub list: Vec<Weapon>,
}

impl Deref for WeaponList {
    type Target = Vec<Weapon>;

    fn deref(&self) -> &Vec<Weapon> {
        &self.list
    }
}

#[derive(Deserialize, Serialize)]
pub struct Weapon {
    pub move_speed: f32,
    pub rate: u32,
    pub recoil: f32,
    pub bullet_sprite: usize,
    pub bullet_spread: f32,
    pub bullet_speed: f32,
    pub bullet_drag: f32,
    pub bullet_bounciness: f32,
    pub bullet_friction: f32,
    pub bullet_collider: (f32, f32),
    pub bullet_timer_limit: u32,
    pub bullet_reflect_limit: u32,
}
