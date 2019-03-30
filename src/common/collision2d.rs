use amethyst::{
    assets::{PrefabData, PrefabError},
    core::nalgebra::*,
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

use crate::common::vector2ext::Vector2Ext;

use std::collections::HashSet;

#[derive(Component, PrefabData, Deserialize, Serialize, Clone, Debug)]
#[prefab(Component)]
#[serde(default)]
pub struct Rigidbody {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub drag: f32,
    pub bounciness: f32,
    pub friction: f32,
    pub auto_rotate: bool,
}
impl Default for Rigidbody {
    fn default() -> Rigidbody {
        Rigidbody {
            velocity: Vector2::zeros(),
            acceleration: Vector2::zeros(),
            drag: 0.0,
            bounciness: 0.0,
            friction: 0.0,
            auto_rotate: false,
        }
    }
}

pub struct RigidbodySystem;
impl<'s> System<'s> for RigidbodySystem {
    type SystemData = (WriteStorage<'s, Transform>, WriteStorage<'s, Rigidbody>);

    fn run(&mut self, (mut transforms, mut rigidbodies): Self::SystemData) {
        for (transform, rigidbody) in (&mut transforms, &mut rigidbodies).join() {
            rigidbody.velocity += rigidbody.acceleration;
            transform.move_global(
                rigidbody
                    .velocity
                    .map(|x| x.max(-5.0).min(5.0))
                    .to_homogeneous(),
            );
            rigidbody.velocity -= rigidbody.velocity * rigidbody.drag;
            if rigidbody.auto_rotate {
                let (_, rad) = rigidbody.velocity.to_polar();
                transform.set_rotation_euler(0.0, 0.0, rad);
            }
        }
    }
}

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
    pub collided: Vec<Collided>,
    pub collision: Vector2<f32>,
}
#[derive(Debug)]
pub struct Collided {
    pub entity: Entity,
    pub tag: String,
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
                        let entry = (tag_a.clone(), tag_b.clone());

                        if self.trigger_entries.contains(&entry) {
                            result_a.collided.push(Collided {
                                entity: ent_b,
                                tag: tag_b,
                            });
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
