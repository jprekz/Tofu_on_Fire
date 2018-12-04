use amethyst::{
    core::cgmath::*,
    core::Transform,
    ecs::prelude::*,
    input::InputHandler,
    renderer::{SpriteRender, SpriteSheetHandle},
    shrev::{EventChannel, ReaderId},
};

use components::*;

pub struct PlayerSystem;
impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        Read<'s, InputHandler<String, String>>,
        Write<'s, EventChannel<GenEvent>>,
        (
            WriteStorage<'s, Player>,
            ReadStorage<'s, Transform>,
            WriteStorage<'s, Rigidbody>,
        ),
    );

    fn run(&mut self, (input, mut gen_event, storages): Self::SystemData) {
        let (mut players, transforms, mut rigidbodies) = storages;
        for (player, transform, rigidbody) in (&mut players, &transforms, &mut rigidbodies).join() {
            let move_x = input.axis_value("move_x").unwrap_or(0.0) as f32;
            let move_y = input.axis_value("move_y").unwrap_or(0.0) as f32;
            rigidbody.acceleration = Vector2::new(move_x, move_y) * player.speed;
            let aim_x = input.axis_value("aim_x").unwrap_or(0.0) as f32;
            let aim_y = input.axis_value("aim_y").unwrap_or(0.0) as f32;
            let shot = input.action_is_down("shot").unwrap();
            if player.trigger_timer > 0 {
                player.trigger_timer -= 1;
            }
            if shot && player.trigger_timer == 0 {
                gen_event.single_write(GenEvent::Bullet {
                    pos: transform.translation.truncate(),
                    vel: Vector2::new(aim_x, aim_y) * 4.0,
                });
                player.trigger_timer = 10;
            }
        }
    }
}

pub enum GenEvent {
    Bullet {
        pos: Vector2<f32>,
        vel: Vector2<f32>,
    },
}
pub struct GeneratorSystem {
    reader: Option<ReaderId<GenEvent>>,
}
impl GeneratorSystem {
    pub fn new() -> GeneratorSystem {
        GeneratorSystem { reader: None }
    }
}
impl<'s> System<'s> for GeneratorSystem {
    type SystemData = (
        Read<'s, EventChannel<GenEvent>>,
        Option<Read<'s, SpriteSheetHandle>>,
        Entities<'s>,
        (
            WriteStorage<'s, Transform>,
            WriteStorage<'s, Rigidbody>,
            WriteStorage<'s, RectCollider<Bullet>>,
            WriteStorage<'s, SpriteRender>,
        ),
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<GenEvent>>().register_reader());
    }

    fn run(&mut self, (gen_event, sheet, entities, storages): Self::SystemData) {
        let sheet = sheet.unwrap();
        let (mut transforms, mut rigidbodies, mut colliders, mut render) = storages;
        for event in gen_event.read(self.reader.as_mut().unwrap()) {
            match event {
                GenEvent::Bullet { pos, vel } => {
                    entities
                        .build_entity()
                        .with(RectCollider::new(4.0, 4.0), &mut colliders)
                        .with(
                            Transform {
                                translation: pos.extend(0.0),
                                ..Default::default()
                            },
                            &mut transforms,
                        ).with(
                            Rigidbody {
                                velocity: *vel,
                                drag: 0.0,
                                bounciness: 1.0,
                                ..Default::default()
                            },
                            &mut rigidbodies,
                        ).with(
                            SpriteRender {
                                sprite_sheet: sheet.clone(),
                                sprite_number: 5,
                                flip_horizontal: false,
                                flip_vertical: false,
                            },
                            &mut render,
                        ).build();
                }
            }
        }
    }
}

pub struct RigidbodySystem;
impl<'s> System<'s> for RigidbodySystem {
    type SystemData = (WriteStorage<'s, Transform>, WriteStorage<'s, Rigidbody>);

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
                    let bounciness = rigidbody.bounciness;
                    rigidbody.velocity -=
                        rigidbody.velocity.dot(normal) * normal * (1.0 + bounciness);
                    transform.translation += a.collision.extend(0.0);
                }
            }
        }
        for (entity, b, transform) in (&entities, &mut b, &mut transforms).join() {
            if let Some(rigidbody) = rigidbodies.get_mut(entity) {
                if !b.collision.is_zero() {
                    let normal = b.collision.normalize();
                    let bounciness = rigidbody.bounciness;
                    rigidbody.velocity -=
                        rigidbody.velocity.dot(normal) * normal * (1.0 + bounciness);
                    transform.translation += b.collision.extend(0.0);
                }
            }
        }
    }
}
