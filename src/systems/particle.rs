use amethyst::ecs::prelude::*;

use crate::components::*;
use crate::skip_fail;

pub struct ParticleSystem;
impl<'s> System<'s> for ParticleSystem {
    type SystemData = (Entities<'s>, WriteStorage<'s, Particle>);

    fn run(&mut self, (entities, mut particles): Self::SystemData) {
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.timer -= 1;
            if particle.timer < 0 {
                skip_fail!(entities.delete(entity));
                continue;
            }
        }
    }
}
