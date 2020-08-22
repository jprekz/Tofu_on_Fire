use amethyst::{core::Hidden, ecs::prelude::*};

use crate::components::*;
use crate::skip_fail;

pub struct ItemSystem;
impl<'s> System<'s> for ItemSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Item>,
        ReadStorage<'s, ColliderResult>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Hidden>,
    );

    fn run(&mut self, (entities, mut items, results, mut players, mut hidden): Self::SystemData) {
        for (entity, mut item, result) in (&entities, &mut items, &results).join() {
            item.timer -= 1;
            if item.timer < 120 {
                if item.timer % 8 <= 4 {
                    let _ = hidden.remove(entity);
                } else {
                    let _ = hidden.insert(entity, Hidden);
                }
            }
            if item.timer < 0 {
                skip_fail!(entities.delete(entity));
                continue;
            }

            let collided_players = result
                .collided
                .iter()
                .filter(|collided| collided.tag == "Player")
                .count();
            if collided_players > 0 {
                for collided in &result.collided {
                    match collided.tag.as_str() {
                        "Player" => {
                            let player = skip_fail!(players
                                .get_mut(collided.entity)
                                .ok_or("Failed to get player component"));
                            player.hp = (player.hp + item.hp).min(100.0);
                        }
                        _ => {}
                    }
                }
                skip_fail!(entities.delete(entity));
            }
        }
    }
}
