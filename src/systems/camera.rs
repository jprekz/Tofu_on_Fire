use amethyst::{core::Transform, ecs::prelude::*, renderer::Camera};

use crate::common::pause::Pause;
use crate::components::*;

#[derive(Default)]
pub struct CameraSystem {
    target_entity: Option<Entity>,
    timer: i32,
}
impl<'s> System<'s> for CameraSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Playable>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Area>,
        Read<'s, Pause>,
    );

    fn run(
        &mut self,
        (entities, cameras, mut transforms, playables, players, areas, pause): Self::SystemData,
    ) {
        if pause.paused() {
            if let Some((_, transform)) = (&areas, &transforms).join().next() {
                let area_x = transform.translation().x;
                let area_y = transform.translation().y;
                for (transform, _) in (&mut transforms, &cameras).join() {
                    let cam_x = transform.translation().x;
                    let cam_y = transform.translation().y;
                    transform.set_translation_x((area_x + cam_x * 9.0) / 10.0);
                    transform.set_translation_y((area_y + cam_y * 9.0) / 10.0);
                }
            }
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
            if self.timer == 0 {
                self.target_entity = None;
            }
        }

        let target_entity = {
            if let Some((entity, _)) = (&entities, &playables).join().next() {
                self.timer = 0;
                entity
            } else if let Some(entity) = self.target_entity {
                entity
            } else if self.timer > 0 {
                return;
            } else if let Some((entity, _)) = (&entities, &players).join().next() {
                entity
            } else {
                return;
            }
        };
        self.target_entity = Some(target_entity);

        let target_pos = {
            if let Some(transform) = transforms.get(target_entity) {
                transform.translation().xy()
            } else {
                self.target_entity = None;
                self.timer = 60;
                return;
            }
        };

        for (transform, _) in (&mut transforms, &cameras).join() {
            transform.set_translation_x(target_pos.x);
            transform.set_translation_y(target_pos.y);
        }
    }
}
