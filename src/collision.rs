use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;
use std::collections::HashSet;

use crate::components::Rigidbody;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
pub struct RectCollider {
    pub tag: String,
    pub width: f32,
    pub height: f32,
}
impl RectCollider {
    pub fn new(tag: impl Into<String>, width: f32, height: f32) -> RectCollider {
        RectCollider {
            tag: tag.into(),
            width,
            height,
        }
    }
}

#[derive(Component, Debug)]
pub struct ColliderResult {
    pub collided: Vec<Entity>,
    pub collision: Vector2<f32>,
}

#[derive(Default)]
pub struct CollisionSystem {
    collide_entries: HashSet<(String, String)>,
    trigger_entries: HashSet<(String, String)>,
}
impl CollisionSystem {
    pub fn collide(mut self, a: impl Into<String>, b: impl Into<String>) -> Self {
        let a = a.into();
        let b = b.into();
        self.collide_entries.insert((a.clone(), b.clone()));
        self.collide_entries.insert((b, a));
        self
    }
    pub fn trigger(mut self, a: impl Into<String>, b: impl Into<String>) -> Self {
        let a = a.into();
        let b = b.into();
        self.trigger_entries.insert((a.clone(), b.clone()));
        self.trigger_entries.insert((b, a));
        self
    }
}
impl<'s> System<'s> for CollisionSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, RectCollider>,
        WriteStorage<'s, ColliderResult>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(&mut self, system_data: Self::SystemData) {
        let (entities, colliders, mut results, mut transforms, mut rigidbodies) = system_data;

        for (ent, _) in (&entities, &colliders).join() {
            let _ = results.insert(
                ent,
                ColliderResult {
                    collided: Vec::new(),
                    collision: Vector2::zeros(),
                },
            );
        }

        (&transforms, &colliders, &mut results).par_join().for_each(
            |(transform_a, collider_a, result_a)| {
                let a_pos = transform_a.translation().xy();
                let a_width = collider_a.width;
                let a_height = collider_a.height;
                let tag_a = collider_a.tag.clone();

                for (ent_b, transform_b, collider_b) in (&entities, &transforms, &colliders).join()
                {
                    let b_pos = transform_b.translation().xy();
                    let b_width = collider_b.width;
                    let b_height = collider_b.height;
                    let tag_b = collider_b.tag.clone();

                    let sub = b_pos - a_pos;
                    let th_x = (a_width + b_width) / 2.0;
                    let th_y = (a_height + b_height) / 2.0;
                    let sinking_x = th_x - sub.x.abs();
                    let sinking_y = th_y - sub.y.abs();
                    if sinking_x > 0.0 && sinking_y > 0.0 {
                        let entry = (tag_a.clone(), tag_b);

                        if self.trigger_entries.contains(&entry) {
                            result_a.collided.push(ent_b);
                        }

                        if !self.collide_entries.contains(&entry) {
                            continue;
                        }

                        if sinking_x < sinking_y {
                            if sub.x > 0.0 {
                                if result_a.collision.x.abs() < sinking_x {
                                    result_a.collision.x = -sinking_x;
                                }
                            } else {
                                if result_a.collision.x.abs() < sinking_x {
                                    result_a.collision.x = sinking_x;
                                }
                            }
                        } else {
                            if sub.y > 0.0 {
                                if result_a.collision.y.abs() < sinking_y {
                                    result_a.collision.y = -sinking_y;
                                }
                            } else {
                                if result_a.collision.y.abs() < sinking_y {
                                    result_a.collision.y = sinking_y;
                                }
                            }
                        }
                    }
                }
            },
        );

        for (result, transform, rigidbody) in
            (&mut results, &mut transforms, &mut rigidbodies).join()
        {
            if result.collision != Vector2::zeros() {
                let normal = result.collision.normalize();
                let bounciness = rigidbody.bounciness;
                let friction = rigidbody.friction;
                rigidbody.velocity -= rigidbody.velocity.dot(&normal) * normal * (1.0 + bounciness);
                rigidbody.velocity *= 1.0 - friction;
                transform.move_global(result.collision.to_homogeneous());
            }
        }
    }
}
