use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct WeaponList {
    pub list: Vec<Weapon>,
}

#[derive(Deserialize, Serialize)]
pub struct Weapon {
    pub move_speed: f32,
    pub rate: u32,
    pub recoil: f32,
    pub bullet_sprite: usize,
    pub bullet_speed: f32,
    pub bullet_drag: f32,
    pub bullet_bounciness: f32,
    pub bullet_collider: (f32, f32),
    pub bullet_timer_limit: u32,
    pub bullet_reflect_limit: u32,
}
