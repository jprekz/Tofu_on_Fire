use amethyst::ecs::prelude::*;

use crate::audio::*;
use crate::components::*;
use crate::skip_fail;

pub struct BulletSystem;
impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Bullet>,
        ReadStorage<'s, ColliderResult>,
        ReadStorage<'s, Player>,
        AudioPlayer<'s>,
    );

    fn run(&mut self, (entities, mut bullets, results, players, mut audio): Self::SystemData) {
        for (entity, bullet, result) in (&entities, &mut bullets, &results).join() {
            if bullet.timer_limit != 0 {
                bullet.timer_count += 1;
                if bullet.timer_count > bullet.timer_limit {
                    skip_fail!(entities.delete(entity));
                    continue;
                }
            }

            for collided in &result.collided {
                match collided.tag.as_str() {
                    "Wall" => {
                        bullet.reflect_count += 1;
                        if bullet.reflect_count > bullet.reflect_limit {
                            skip_fail!(entities.delete(entity));
                        }
                    }
                    "Player" => {
                        let player = skip_fail!(players
                            .get(collided.entity)
                            .ok_or("Failed to get player component"));
                        if player.team != bullet.team {
                            if !bullet.pierce {
                                skip_fail!(entities.delete(entity));
                            }
                            audio.play_once(entity, 3, 0.2 + bullet.damage / 25.0);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
