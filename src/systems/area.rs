use amethyst::{core::math::*, core::Transform, ecs::prelude::*, renderer::SpriteRender};

use crate::components::*;
use crate::resources::Score;
use crate::skip_fail;

#[derive(Default)]
pub struct AreaSystem {
    timer: i32,
}
impl<'s> System<'s> for AreaSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Area>,
        ReadStorage<'s, AreaTarget>,
        ReadStorage<'s, ColliderResult>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteExpect<'s, Score>,
    );

    fn run(
        &mut self,
        (players, areas, targets, results, mut transforms, mut sprites, mut score): Self::SystemData,
    ) {
        self.timer += 1;

        if self.timer % 2 == 0 {
            for (_, transform) in (&targets, &mut transforms).join() {
                transform.append_rotation_z_axis(f32::pi());
            }
        }

        if self.timer % 60 != 0 {
            return;
        }

        for (_, result, transform, sprite) in
            (&areas, &results, &mut transforms, &mut sprites).join()
        {
            let mut p = 0i32;
            for collided in &result.collided {
                let player = skip_fail!(players
                    .get(collided.entity)
                    .ok_or("Failed to get player component"));
                let team = player.team;
                score.score[team as usize] += 1;
                match team {
                    0 => p += 1,
                    1 => p -= 1,
                    _ => (),
                };
            }
            let position = score.score[0] as i32 - score.score[1] as i32;
            let ratio = position as f32 / 100.0 + 0.5;
            let ratio = if ratio < 0.0 {
                0.0
            } else if ratio > 1.0 {
                1.0
            } else {
                ratio
            };
            let position_x = 352.0 * ratio + 176.0;
            transform.set_translation_x(position_x);
            sprite.sprite_number = match p.cmp(&0) {
                std::cmp::Ordering::Equal => 16,
                std::cmp::Ordering::Less => 18,
                std::cmp::Ordering::Greater => 17,
            }
        }
    }
}
