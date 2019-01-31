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

    #[serde(skip)]
    pub collided: Vec<Entity>,
    #[serde(skip, default = "Vector2::zeros")]
    pub collision: Vector2<f32>,
}
impl RectCollider {
    pub fn new(tag: impl Into<String>, width: f32, height: f32) -> RectCollider {
        RectCollider {
            tag: tag.into(),
            width,
            height,
            collided: Vec::new(),
            collision: Vector2::zeros(),
        }
    }
}

#[derive(Default)]
pub struct CollisionSystem {
    collide_entries: HashSet<(String, String)>,
    trigger_entries: HashSet<(String, String)>,
    collided_changeset: Vec<(Entity, Entity)>,
    collision_x_changeset: ChangeSet<AddAssignAbsMax>,
    collision_y_changeset: ChangeSet<AddAssignAbsMax>,
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
        WriteStorage<'s, RectCollider>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(&mut self, system_data: Self::SystemData) {
        let (entities, mut colliders, mut transforms, mut rigidbodies) = system_data;

        let Self {
            collide_entries,
            trigger_entries,
            collided_changeset,
            collision_x_changeset,
            collision_y_changeset,
        } = self;

        collided_changeset.clear();
        collision_x_changeset.clear();
        collision_y_changeset.clear();

        for collider in (&mut colliders).join() {
            collider.collided.clear();
            collider.collision = Vector2::zeros();
        }

        for (ent_a, collider_a, transform_a) in (&entities, &colliders, &transforms).join() {
            let a_size = Vector2::new(collider_a.width, collider_a.height);
            let a_pos: Vector2<f32> = transform_a.translation().xy().into();
            for (ent_b, collider_b, transform_b) in (&entities, &colliders, &transforms).join() {
                if ent_a.id() >= ent_b.id() {
                    continue;
                }
                let b_size = Vector2::new(collider_b.width, collider_b.height);
                let b_pos: Vector2<f32> = transform_b.translation().xy().into();
                let sub = b_pos - a_pos;
                let sinking = (a_size / 2.0 + b_size / 2.0) - sub.map(f32::abs);
                if sinking.x > 0.0 && sinking.y > 0.0 {
                    let tag_a = collider_a.tag.clone();
                    let tag_b = collider_b.tag.clone();
                    let entry = (tag_a, tag_b);

                    if trigger_entries.contains(&entry) {
                        collided_changeset.push((ent_a, ent_b));
                        collided_changeset.push((ent_b, ent_a));
                    }

                    if !collide_entries.contains(&entry) {
                        continue;
                    }

                    if sinking.x < sinking.y {
                        if sub.x > 0.0 {
                            collision_x_changeset.add(ent_a, (-sinking.x).into());
                            collision_x_changeset.add(ent_b, (sinking.x).into());
                        } else {
                            collision_x_changeset.add(ent_a, (sinking.x).into());
                            collision_x_changeset.add(ent_b, (-sinking.x).into());
                        }
                    } else {
                        if sub.y > 0.0 {
                            collision_y_changeset.add(ent_a, (-sinking.y).into());
                            collision_y_changeset.add(ent_b, (sinking.y).into());
                        } else {
                            collision_y_changeset.add(ent_a, (sinking.y).into());
                            collision_y_changeset.add(ent_b, (-sinking.y).into());
                        }
                    }
                }
            }
        }

        for (ref mut collider, ref modifier) in (&mut colliders, collision_x_changeset).join() {
            collider.collision.x = modifier.0;
        }
        for (ref mut collider, ref modifier) in (&mut colliders, collision_y_changeset).join() {
            collider.collision.y = modifier.0;
        }

        for (a, b) in collided_changeset {
            colliders.get_mut(*a).unwrap().collided.push(*b);
        }

        for (collider, transform, rigidbody) in
            (&mut colliders, &mut transforms, &mut rigidbodies).join()
        {
            if collider.collision != Vector2::zeros() {
                let normal = collider.collision.normalize();
                let bounciness = rigidbody.bounciness;
                let friction = rigidbody.friction;
                rigidbody.velocity -= rigidbody.velocity.dot(&normal) * normal * (1.0 + bounciness);
                rigidbody.velocity *= 1.0 - friction;
                transform.move_global(collider.collision.to_homogeneous());
            }
        }
    }
}

#[derive(Clone, Copy)]
struct AddAssignAbsMax(f32);
impl std::ops::AddAssign for AddAssignAbsMax {
    fn add_assign(&mut self, other: Self) {
        if self.0.abs() < other.0.abs() {
            self.0 = other.0
        };
    }
}
impl Into<AddAssignAbsMax> for f32 {
    fn into(self) -> AddAssignAbsMax {
        AddAssignAbsMax(self)
    }
}
