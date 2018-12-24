use amethyst::{
    core::nalgebra::*,
    core::Transform,
    ecs::prelude::*,
    input::InputHandler,
    shrev::{EventChannel, ReaderId},
};

use crate::components::*;
use crate::prefab::*;

pub use crate::collision::CollisionSystem;

trait Vector2Ext<N> {
    fn to_polar(&self) -> (N, N);
    fn from_polar(r: N, theta: N) -> Self;
}
impl<N: Real> Vector2Ext<N> for Vector2<N> {
    fn to_polar(&self) -> (N, N) {
        (self.x.hypot(self.y), self.y.atan2(self.x))
    }
    fn from_polar(r: N, theta: N) -> Self {
        Vector2::new(theta.cos() * r, theta.sin() * r)
    }
}

pub struct PlayableSystem;
impl<'s> System<'s> for PlayableSystem {
    type SystemData = (
        Read<'s, InputHandler<String, String>>,
        ReadStorage<'s, Playable>,
        WriteStorage<'s, Player>,
    );

    fn run(&mut self, (input, playables, mut players): Self::SystemData) {
        let axis_xy_value =
            |x: &str, y: &str| Some(Vector2::new(input.axis_value(x)?, input.axis_value(y)?));

        let move_vec = axis_xy_value("move_x", "move_y").unwrap();
        let left_vec = axis_xy_value("left_x", "left_y").unwrap();
        let move_vec = move_vec + left_vec;
        let (move_r, move_theta) = move_vec.to_polar();
        let move_vec = Vector2::from_polar(move_r.min(1.0), move_theta);
        let move_vec = move_vec.map(|v| v as f32);

        let aim_vec = axis_xy_value("aim_x", "aim_y").unwrap();
        let right_vec = axis_xy_value("right_x", "right_y").unwrap();
        let aim_vec = aim_vec + right_vec;
        let (aim_r, aim_theta) = aim_vec.to_polar();
        let aim_vec = Vector2::from_polar(aim_r.min(1.0), aim_theta);
        let aim_vec = aim_vec.map(|v| v as f32);

        let shot = input.action_is_down("shot").unwrap();

        for (_, player) in (&playables, &mut players).join() {
            player.input_move = move_vec;
            player.input_aim = aim_vec;
            player.input_shot = shot;
        }
    }
}

pub struct AISystem;
impl<'s> System<'s> for AISystem {
    type SystemData = (
        ReadStorage<'s, AI>,
        ReadStorage<'s, Playable>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (ai, playables, mut players, transforms): Self::SystemData) {
        let mut target = Vector2::zeros();
        for (_, transform) in (&playables, &transforms).join() {
            target = transform.translation().xy();
        }

        for (_, player, transform) in (&ai, &mut players, &transforms).join() {
            let pos = transform.translation().xy();
            let dist = target - pos;
            let move_vec = if dist != Vector2::zeros() {
                dist.normalize()
            } else {
                Vector2::zeros()
            };

            player.input_move = move_vec;
            player.input_shot = true;
        }
    }
}

pub struct PlayerSystem;
impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        Write<'s, EventChannel<MyPrefabData>>,
        (
            WriteStorage<'s, Player>,
            ReadStorage<'s, Transform>,
            WriteStorage<'s, Rigidbody>,
        ),
    );

    fn run(&mut self, (mut prefab_data_loader, storages): Self::SystemData) {
        let (mut players, transforms, mut rigidbodies) = storages;
        for (player, transform, rigidbody) in (&mut players, &transforms, &mut rigidbodies).join() {
            let move_vec = player.input_move;
            let aim_vec = player.input_aim;
            let aim_r = aim_vec.x.hypot(aim_vec.y);
            let shot = player.input_shot;

            rigidbody.acceleration = move_vec * player.speed;

            if player.trigger_timer > 0 {
                player.trigger_timer -= 1;
            }
            if shot && player.trigger_timer == 0 {
                let bullet_vel = if aim_r < 0.1 { move_vec } else { aim_vec };
                prefab_data_loader.single_write(MyPrefabData {
                    transform: Some(transform.clone()),
                    rigidbody: Some(Rigidbody {
                        velocity: bullet_vel * 4.0,
                        bounciness: 0.8,
                        auto_rotate: true,
                        ..Default::default()
                    }),
                    sprite: Some(SpriteRenderPrefab { sprite_number: 4 }),
                    collider: Some(RectCollider::new("Bullet", 4.0, 4.0)),
                    bullet: Some(Bullet::new(player.team, 120, 3)),
                    ..Default::default()
                });
                player.trigger_timer = 10;
                rigidbody.acceleration = -bullet_vel * 500.0;
            }
            if player.knock_back != Vector2::zeros() {
                rigidbody.acceleration = player.knock_back * 500.0;
                player.damage = 0;
                player.knock_back = Vector2::zeros();
            }
        }
    }
}

pub struct BulletSystem;
impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Bullet>,
        ReadStorage<'s, RectCollider>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (entities, mut bullets, colliders, players): Self::SystemData) {
        for (entity, bullet, collider) in (&entities, &mut bullets, &colliders).join() {
            if bullet.timer_limit != 0 {
                bullet.timer_count += 1;
                if bullet.timer_count > bullet.timer_limit {
                    entities.delete(entity).unwrap();
                }
            }

            for &collided in &collider.collided {
                match colliders.get(collided).unwrap().tag.as_str() {
                    "Wall" => {
                        bullet.reflect_count += 1;
                        if bullet.reflect_count > bullet.reflect_limit {
                            entities.delete(entity).unwrap();
                        }
                    }
                    "Player" => {
                        if players.get(collided).unwrap().team != bullet.team {
                            entities.delete(entity).unwrap();
                        }
                    }
                    _ => {}
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
            transform.move_global(
                rigidbody
                    .velocity
                    .map(|x| x.max(-3.0).min(3.0))
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

#[derive(Default)]
pub struct PrefabDataLoaderSystem<T: 'static> {
    reader: Option<ReaderId<T>>,
}
impl<'s, T> System<'s> for PrefabDataLoaderSystem<T>
where
    T: amethyst::assets::PrefabData<'s> + Send + Sync + 'static,
{
    type SystemData = (Entities<'s>, Read<'s, EventChannel<T>>, T::SystemData);

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<T>>().register_reader());
    }

    fn run(&mut self, (entities, channel, mut prefab_system_data): Self::SystemData) {
        for prefab_data in channel.read(self.reader.as_mut().unwrap()) {
            let entity = entities.create();
            prefab_data
                .add_to_entity(entity, &mut prefab_system_data, &[entity])
                .expect("Unable to add prefab system data to entity");
        }
    }
}
