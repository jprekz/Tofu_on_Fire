use amethyst::{core::math::*, core::transform::*, core::Transform, ecs::prelude::*};

use crate::common::vector2ext::Vector2Ext;
use crate::components::*;
use crate::skip_fail;

pub struct ReticleSystem;
impl<'s> System<'s> for ReticleSystem {
    type SystemData = (
        ReadStorage<'s, Reticle>,
        ReadStorage<'s, ReticleLine>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (reticles, lines, parents, mut transforms, players): Self::SystemData) {
        for (_, parent, transform) in (&reticles, &parents, &mut transforms).join() {
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
            let aim_vec = player.input_aim * 100.0;
            transform.set_translation_x(aim_vec.x);
            transform.set_translation_y(aim_vec.y);
        }
        for (_, parent, transform) in (&lines, &parents, &mut transforms).join() {
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
            let aim_vec = player.input_aim * 100.0;
            let (l, rad) = aim_vec.to_polar();
            transform.set_rotation_euler(0.0, 0.0, rad);
            transform.set_scale(Vector3::new(l / 100.0, 1.0, 1.0));
        }
    }
}
