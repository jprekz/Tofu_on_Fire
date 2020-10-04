use amethyst::{
    assets::PrefabData, core::math::*, core::Transform, derive::PrefabData, ecs::prelude::*, Error,
};
use serde_derive::{Deserialize, Serialize};
use specs_derive::Component;

use crate::common::quadtreegrid::QuadTreeGrid;
use crate::common::vector2ext::Vector2Ext;

use std::collections::{HashMap, HashSet};

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
            transform.prepend_translation(
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

pub struct CollisionSystem {
    collide_entries: HashSet<(String, String)>,
    trigger_entries: HashSet<(String, String)>,
    bitsets: HashMap<String, BitSet>,
    qtg: QuadTreeGrid<(Entity, bool)>,
}
impl Default for CollisionSystem {
    fn default() -> Self {
        Self {
            collide_entries: HashSet::new(),
            trigger_entries: HashSet::new(),
            bitsets: HashMap::new(),
            qtg: QuadTreeGrid::new(4, 0.0, 0.0, 640.0, 480.0),
        }
    }
}
impl CollisionSystem {
    pub fn collide(mut self, a: impl Into<String>, b: impl Into<String>) -> Self {
        let a = a.into();
        let b = b.into();
        self.collide_entries.insert((a.clone(), b.clone()));
        self.collide_entries.insert((b.clone(), a.clone()));
        self.bitsets.insert(a, BitSet::new());
        self.bitsets.insert(b, BitSet::new());
        self
    }
    pub fn trigger(mut self, a: impl Into<String>, b: impl Into<String>) -> Self {
        let a = a.into();
        let b = b.into();
        self.trigger_entries.insert((a.clone(), b.clone()));
        self.trigger_entries.insert((b.clone(), a.clone()));
        self.bitsets.insert(a, BitSet::new());
        self.bitsets.insert(b, BitSet::new());
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

        for (_, bitset) in self.bitsets.iter_mut() {
            bitset.clear();
        }

        for (ent, collider) in (&entities, &colliders).join() {
            if results.contains(ent) {
                let result = results.get_mut(ent).unwrap();
                result.collided.clear();
                result.collision.fill(0.0);
            } else {
                let _ = results.insert(
                    ent,
                    ColliderResult {
                        collided: Vec::new(),
                        collision: Vector2::zeros(),
                    },
                );
            }

            self.bitsets.get_mut(&collider.tag).unwrap().add(ent.id());
        }

        let entries = self.collide_entries.union(&self.trigger_entries);

        for entry in entries {
            let (tag_a, tag_b) = entry;
            let bitset_a = self.bitsets.get(tag_a).unwrap();
            let bitset_b = self.bitsets.get(tag_b).unwrap();
            let is_trigger = self.trigger_entries.contains(&entry);
            let is_collide = self.collide_entries.contains(&entry);

            for (_, entity, collider, transform) in
                (bitset_a, &entities, &colliders, &transforms).join()
            {
                let x = transform.translation().x as f64;
                let y = transform.translation().y as f64;
                let w = collider.width as f64;
                let h = collider.height as f64;
                self.qtg
                    .add_entity(x - w / 2.0, y - h / 2.0, w, h, (entity, true));
            }

            for (_, entity, collider, transform) in
                (bitset_b, &entities, &colliders, &transforms).join()
            {
                let x = transform.translation().x as f64;
                let y = transform.translation().y as f64;
                let w = collider.width as f64;
                let h = collider.height as f64;
                self.qtg
                    .add_entity(x - w / 2.0, y - h / 2.0, w, h, (entity, false));
            }

            self.qtg.iter_entity_pair(|(ent_a, a), (ent_b, b)| {
                if a == b {
                    return;
                }
                let (ent_a, ent_b) = if *a {
                    (*ent_a, *ent_b)
                } else {
                    (*ent_b, *ent_a)
                };

                let transform_a = transforms.get(ent_a).unwrap();
                let collider_a = colliders.get(ent_a).unwrap();
                let a_pos = transform_a.translation().xy();
                let a_width = collider_a.width;
                let a_height = collider_a.height;

                let transform_b = transforms.get(ent_b).unwrap();
                let collider_b = colliders.get(ent_b).unwrap();
                let b_pos = transform_b.translation().xy();
                let b_width = collider_b.width;
                let b_height = collider_b.height;

                let sub = b_pos - a_pos;
                let th_x = (a_width + b_width) / 2.0;
                let th_y = (a_height + b_height) / 2.0;
                let sinking_x = th_x - sub.x.abs();
                let sinking_y = th_y - sub.y.abs();

                if !(sinking_x > 0.0 && sinking_y > 0.0) {
                    return;
                }

                if is_trigger {
                    results.get_mut(ent_a).unwrap().collided.push(Collided {
                        entity: ent_b,
                        tag: tag_b.clone(),
                    });
                    results.get_mut(ent_b).unwrap().collided.push(Collided {
                        entity: ent_a,
                        tag: tag_a.clone(),
                    });
                }

                if is_collide {
                    if sinking_x < sinking_y {
                        if sub.x > 0.0 {
                            if let Some(result_a) = results.get_mut(ent_a) {
                                if result_a.collision.x.abs() < sinking_x {
                                    result_a.collision.x = -sinking_x;
                                }
                            }
                            if let Some(result_b) = results.get_mut(ent_b) {
                                if result_b.collision.x.abs() < sinking_x {
                                    result_b.collision.x = sinking_x;
                                }
                            }
                        } else {
                            if let Some(result_a) = results.get_mut(ent_a) {
                                if result_a.collision.x.abs() < sinking_x {
                                    result_a.collision.x = sinking_x;
                                }
                            }
                            if let Some(result_b) = results.get_mut(ent_b) {
                                if result_b.collision.x.abs() < sinking_x {
                                    result_b.collision.x = -sinking_x;
                                }
                            }
                        }
                    } else {
                        if sub.y > 0.0 {
                            if let Some(result_a) = results.get_mut(ent_a) {
                                if result_a.collision.y.abs() < sinking_y {
                                    result_a.collision.y = -sinking_y;
                                }
                            }
                            if let Some(result_b) = results.get_mut(ent_b) {
                                if result_b.collision.y.abs() < sinking_y {
                                    result_b.collision.y = sinking_y;
                                }
                            }
                        } else {
                            if let Some(result_a) = results.get_mut(ent_a) {
                                if result_a.collision.y.abs() < sinking_y {
                                    result_a.collision.y = sinking_y;
                                }
                            }
                            if let Some(result_b) = results.get_mut(ent_b) {
                                if result_b.collision.y.abs() < sinking_y {
                                    result_b.collision.y = -sinking_y;
                                }
                            }
                        }
                    }
                }
            });
        }

        for (result, transform, rigidbody) in (&results, &mut transforms, &mut rigidbodies).join() {
            if result.collision != Vector2::zeros() {
                let normal = result.collision.normalize();
                let bounciness = rigidbody.bounciness;
                let friction = rigidbody.friction;
                rigidbody.velocity -= rigidbody.velocity.dot(&normal) * normal * (1.0 + bounciness);
                rigidbody.velocity *= 1.0 - friction;
                transform.prepend_translation(result.collision.to_homogeneous());
            }
        }
    }
}
