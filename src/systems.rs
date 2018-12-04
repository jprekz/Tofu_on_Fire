use amethyst::{
    core::cgmath::*,
    core::Transform,
    ecs::prelude::*,
    input::InputHandler,
};

use components::*;

pub struct PlayerSystem;
impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        Read<'s, InputHandler<String, String>>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(&mut self, (input, players, mut rigidbodies): Self::SystemData) {
        for (player, rigidbody) in (&players, &mut rigidbodies).join() {
            let input_x = input.axis_value("move_x").unwrap_or(0.0) as f32;
            let input_y = input.axis_value("move_y").unwrap_or(0.0) as f32;
            rigidbody.acceleration = Vector2::new(input_x, input_y) * player.speed;
        }
    }
}

pub struct RigidbodySystem;
impl<'s> System<'s> for RigidbodySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(&mut self, (mut transforms, mut rigidbodies): Self::SystemData) {
        for (transform, rigidbody) in (&mut transforms, &mut rigidbodies).join() {
            rigidbody.velocity += rigidbody.acceleration;
            transform.translation += rigidbody.velocity.extend(0.0);
            rigidbody.velocity -= rigidbody.velocity * rigidbody.drag;
        }
    }
}

use std::marker::PhantomData;
pub struct CollisionSystem<A, B>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    a: PhantomData<A>,
    b: PhantomData<B>,
}
impl<A, B> CollisionSystem<A, B>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    pub fn new() -> CollisionSystem<A, B> {
        CollisionSystem {
            a: PhantomData,
            b: PhantomData,
        }
    }
}
impl<'s, A, B> System<'s> for CollisionSystem<A, B>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, RectCollider<A>>,
        WriteStorage<'s, RectCollider<B>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(&mut self, (entities, mut a, mut b, mut transforms, mut rigidbodies): Self::SystemData) {
        for a in (&mut a).join() {
            a.collision = Vector2::<f32>::zero();
        }
        for b in (&mut b).join() {
            b.collision = Vector2::<f32>::zero();
        }
        for (a, a_transform) in (&mut a, &transforms).join() {
            let a_size = Vector2::new(a.width, a.height);
            let a_pos = a_transform.translation.truncate();
            for (b, b_transform) in (&mut b, &transforms).join() {
                let b_size = Vector2::new(b.width, b.height);
                let b_pos = b_transform.translation.truncate();
                let sub = b_pos - a_pos;
                let sinking = (a_size / 2.0 + b_size / 2.0) - sub.map(f32::abs);
                if sinking.x > 0.0 && sinking.y > 0.0 {
                    if sinking.x < sinking.y {
                        if sub.x > 0.0 {
                            a.collision.x = -sinking.x;
                            b.collision.x = sinking.x;
                        } else {
                            a.collision.x = sinking.x;
                            b.collision.x = -sinking.x;
                        }
                    } else {
                        if sub.y > 0.0 {
                            a.collision.y = -sinking.y;
                            b.collision.y = sinking.y;
                        } else {
                            a.collision.y = sinking.y;
                            b.collision.y = -sinking.y;
                        }
                    }
                }
            }
        }
        for (entity, a, transform) in (&entities, &mut a, &mut transforms).join() {
            if let Some(rigidbody) = rigidbodies.get_mut(entity) {
                if !a.collision.is_zero() {
                    let normal = a.collision.normalize();
                    rigidbody.velocity -= rigidbody.velocity.dot(normal) * normal;
                    transform.translation += a.collision.extend(0.0);
                }
            }
        }
        for (entity, b, transform) in (&entities, &mut b, &mut transforms).join() {
            if let Some(rigidbody) = rigidbodies.get_mut(entity) {
                if !b.collision.is_zero() {
                    let normal = b.collision.normalize();
                    rigidbody.velocity -= rigidbody.velocity.dot(normal) * normal;
                    transform.translation += b.collision.extend(0.0);
                }
            }
        }
    }
}
