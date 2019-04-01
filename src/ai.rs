use crate::common::vector2ext::Vector2Ext;
use crate::components::*;
use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
};
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct AI {
    #[serde(skip)]
    pub state: AIState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AIState {
    Go(Entity),
    Back(Entity),
    Right(Entity),
    Left(Entity),
    Neutral,
}
impl Default for AIState {
    fn default() -> AIState {
        AIState::Neutral
    }
}
impl AI {
    fn target(&self) -> Option<Entity> {
        match self.state {
            AIState::Go(target) => Some(target),
            AIState::Back(target) => Some(target),
            AIState::Right(target) => Some(target),
            AIState::Left(target) => Some(target),
            _ => None,
        }
    }
}

pub struct AISystem;
impl<'s> System<'s> for AISystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, AI>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Rigidbody>,
        ReadStorage<'s, Bullet>,
    );

    fn run(
        &mut self,
        (entities, mut ai, mut players, transforms, rigidbodies, bullets): Self::SystemData,
    ) {
        for (entity, ai, transform, rigidbody) in
            (&entities, &mut ai, &transforms, &rigidbodies).join()
        {
            let mut rng = thread_rng();

            let my_team = match players.get(entity) {
                Some(player) => player.team,
                None => {
                    log::warn!("Failed to get player component");
                    continue;
                }
            };
            let my_pos = transform.translation().xy();

            // change target
            if rng.gen_bool(0.01) || ai.state == AIState::Neutral {
                if let Some((next_target, _, _)) = (&entities, &players, &transforms)
                    .join()
                    .filter(|(_, target, _)| target.team != my_team)
                    .min_by_key(|(_, _, transform)| {
                        (transform.translation().xy() - my_pos).norm() as i32
                    })
                {
                    ai.state = AIState::Go(next_target);
                }
            }

            if rng.gen_bool(0.1) && rigidbody.velocity.norm() < 0.1 {
                if let Some(target) = ai.target() {
                    if rng.gen_bool(0.5) {
                        ai.state = AIState::Right(target);
                    } else {
                        ai.state = AIState::Left(target);
                    }
                }
            }

            if rng.gen_bool(0.1) {
                if let Some((next_target, _, _)) = (&entities, &bullets, &transforms)
                    .join()
                    .filter(|(_, bullet, _)| bullet.team != my_team)
                    .filter(|(_, _, transform)| {
                        (transform.translation().xy() - my_pos).norm() < 40.0
                    })
                    .min_by_key(|(_, _, transform)| {
                        (transform.translation().xy() - my_pos).norm() as i32
                    })
                {
                    ai.state = AIState::Back(next_target);
                }
            }

            let normalize = |a: Vector2<f32>| {
                if a != Vector2::zeros() {
                    a.normalize()
                } else {
                    Vector2::zeros()
                }
            };

            let (input_move, input_shot) = match ai.state.clone() {
                AIState::Go(target) => {
                    let target_pos = if let Some(t) = transforms.get(target) {
                        t.translation().xy()
                    } else {
                        ai.state = AIState::Neutral;
                        continue;
                    };
                    let dist = target_pos - my_pos;
                    let mut move_vec = normalize(dist);

                    if dist.norm() > 40.0 {
                        let (r, theta) = dist.to_polar();
                        let theta = (theta / f32::frac_pi_4()).round() * f32::frac_pi_4();
                        move_vec = Vector2::from_polar(r, theta).normalize();
                    }

                    (move_vec, true)
                }
                AIState::Back(target) => {
                    let target_pos = if let Some(t) = transforms.get(target) {
                        t.translation().xy()
                    } else {
                        ai.state = AIState::Neutral;
                        continue;
                    };
                    let dist = target_pos - my_pos;
                    let move_vec = -normalize(dist);

                    (move_vec, true)
                }
                AIState::Right(target) => {
                    let target_pos = if let Some(t) = transforms.get(target) {
                        t.translation().xy()
                    } else {
                        ai.state = AIState::Neutral;
                        continue;
                    };
                    let dist = target_pos - my_pos;
                    let move_vec = Rotation2::new(Real::frac_pi_2()) * normalize(dist);

                    (move_vec, true)
                }
                AIState::Left(target) => {
                    let target_pos = if let Some(t) = transforms.get(target) {
                        t.translation().xy()
                    } else {
                        ai.state = AIState::Neutral;
                        continue;
                    };
                    let dist = target_pos - my_pos;
                    let move_vec = Rotation2::new(Real::frac_pi_2()).inverse() * normalize(dist);

                    (move_vec, true)
                }
                _ => (Vector2::zeros(), false),
            };

            match players.get_mut(entity) {
                Some(player) => {
                    player.input_move = input_move;
                    player.input_shot = input_shot;
                }
                None => {
                    log::warn!("Failed to get player component");
                    continue;
                }
            };
        }
    }
}
