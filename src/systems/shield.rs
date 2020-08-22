use amethyst::{core::transform::*, ecs::prelude::*, renderer::SpriteRender};

use crate::components::*;
use crate::skip_fail;

pub struct ShieldSystem;
impl<'s> System<'s> for ShieldSystem {
    type SystemData = (
        ReadStorage<'s, Shield>,
        WriteStorage<'s, SpriteRender>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (shields, mut renders, parents, players): Self::SystemData) {
        for (_, parent, render) in (&shields, &parents, &mut renders).join() {
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
            let hp = player.hp;
            if hp >= 100.0 {
                render.sprite_number = 10;
            } else if hp > 66.6 {
                render.sprite_number = 11;
            } else if hp > 33.3 {
                render.sprite_number = 12;
            } else {
                render.sprite_number = 13;
            }
        }
    }
}
