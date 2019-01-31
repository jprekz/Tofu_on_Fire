use amethyst::{
    core::nalgebra::*,
    core::transform::components::Parent,
    core::Transform,
    ecs::prelude::*,
    input::InputHandler,
    shrev::{EventChannel, ReaderId},
};
use rand::{distributions::*, prelude::*};

use crate::components::*;
use crate::prefab::*;
use crate::weapon::*;

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
        WriteStorage<'s, Playable>,
        WriteStorage<'s, Player>,
    );

    fn run(&mut self, (input, mut playables, mut players): Self::SystemData) {
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
        let change = input.action_is_down("change").unwrap();

        for (playable, player) in (&mut playables, &mut players).join() {
            player.input_move = move_vec;
            player.input_aim = aim_vec;
            player.input_shot = shot;
            player.input_change = change && !playable.input_change_hold;
            playable.input_change_hold = change;
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
    );

    fn run(&mut self, (entities, mut ai, mut players, transforms): Self::SystemData) {
        for (entity, ai, transform) in (&entities, &mut ai, &transforms).join() {
            let mut rng = thread_rng();
            if ai.target.is_none() || rng.gen_bool(0.01) {
                let my_team = players.get(entity).unwrap().team;
                ai.target = (&entities, &players)
                    .join()
                    .filter(|(_, target)| target.team != my_team)
                    .collect::<Vec<_>>()
                    .choose(&mut rng)
                    .map(|(entity, _)| *entity);
            }

            if let Some(target) = ai.target {
                let my_pos = transform.translation().xy();
                let target_pos = transforms.get(target).unwrap().translation().xy();
                let dist = target_pos - my_pos;
                let move_vec = if dist != Vector2::zeros() {
                    dist.normalize()
                } else {
                    Vector2::zeros()
                };

                let player = players.get_mut(entity).unwrap();
                player.input_move = move_vec;
                player.input_shot = true;
            }
        }
    }
}

pub struct PlayerSystem;
impl<'s> System<'s> for PlayerSystem {
    type SystemData = (
        Write<'s, EventChannel<MyPrefabData>>,
        Read<'s, WeaponList>,
        (
            WriteStorage<'s, Player>,
            ReadStorage<'s, Bullet>,
            ReadStorage<'s, Transform>,
            WriteStorage<'s, Rigidbody>,
            ReadStorage<'s, RectCollider>,
            ReadStorage<'s, ColliderResult>,
        ),
    );

    fn run(&mut self, (mut prefab_data_loader, weapon_list, storages): Self::SystemData) {
        let (mut players, bullets, transforms, mut rigidbodies, colliders, results) = storages;

        for (player, transform, rigidbody, result) in
            (&mut players, &transforms, &mut rigidbodies, &results).join()
        {
            let weapon = &weapon_list[player.weapon];

            let move_vec = player.input_move;
            let aim_vec = player.input_aim;
            let aim_r = aim_vec.x.hypot(aim_vec.y);
            let shot = player.input_shot;
            let change = player.input_change;

            rigidbody.acceleration = move_vec * weapon.move_speed;

            if change {
                player.weapon += 1;
                if player.weapon >= weapon_list.len() {
                    player.weapon = 0;
                }
            }

            if player.trigger_timer > 0 {
                player.trigger_timer -= 1;
            }
            if shot && player.trigger_timer == 0 {
                let bullet_vel = if aim_r < 0.1 { move_vec } else { aim_vec };
                let (r, theta) = bullet_vel.to_polar();
                let spread = Uniform::new_inclusive(-weapon.bullet_spread, weapon.bullet_spread)
                    .sample(&mut thread_rng());
                let bullet_vel = Vector2::from_polar(r, theta + spread);

                let mut bullet_transform = transform.clone();
                bullet_transform.set_z(-1.0);

                prefab_data_loader.single_write(MyPrefabData {
                    transform: Some(bullet_transform),
                    rigidbody: Some(Rigidbody {
                        velocity: bullet_vel * weapon.bullet_speed,
                        drag: weapon.bullet_drag,
                        bounciness: weapon.bullet_bounciness,
                        friction: weapon.bullet_friction,
                        auto_rotate: true,
                        ..Default::default()
                    }),
                    sprite: Some(SpriteRenderPrefab {
                        sprite_number: weapon.bullet_sprite,
                    }),
                    collider: Some(RectCollider::new(
                        "Bullet",
                        weapon.bullet_collider.0,
                        weapon.bullet_collider.1,
                    )),
                    bullet: Some(Bullet::new(
                        player.team,
                        weapon.bullet_timer_limit,
                        weapon.bullet_reflect_limit,
                        weapon.bullet_knockback,
                        weapon.bullet_slowing,
                        weapon.bullet_pierce,
                    )),
                    ..Default::default()
                });
                player.trigger_timer = weapon.rate;
            }

            for &collided in &result.collided {
                let bullet = bullets.get(collided).unwrap();
                match colliders.get(collided).unwrap().tag.as_str() {
                    "Bullet" if bullet.team != player.team => {
                        let b_pos = transforms.get(collided).unwrap().translation().xy();
                        let p_pos = transform.translation().xy();
                        let dist = p_pos - b_pos;
                        rigidbody.velocity *= 1.0 - bullet.slowing;
                        rigidbody.acceleration *= 1.0 - bullet.slowing;
                        rigidbody.acceleration +=
                            dist.try_normalize(0.0).unwrap_or(Vector2::zeros()) * bullet.knockback;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct ReticleSystem;
impl<'s> System<'s> for ReticleSystem {
    type SystemData = (
        ReadStorage<'s, Reticle>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (reticles, parents, mut transforms, players): Self::SystemData) {
        for (_, parent, transform) in (&reticles, &parents, &mut transforms).join() {
            let player = players.get(parent.entity).unwrap();
            let move_vec = player.input_move;
            let aim_vec = player.input_aim;
            let aim_r = aim_vec.x.hypot(aim_vec.y);
            let v = if aim_r < 0.1 { move_vec } else { aim_vec } * 100.0;
            transform.set_x(v.x);
            transform.set_y(v.y);
        }
    }
}

pub struct BulletSystem;
impl<'s> System<'s> for BulletSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Bullet>,
        ReadStorage<'s, RectCollider>,
        ReadStorage<'s, ColliderResult>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (entities, mut bullets, colliders, results, players): Self::SystemData) {
        for (entity, bullet, result) in (&entities, &mut bullets, &results).join() {
            if bullet.timer_limit != 0 {
                bullet.timer_count += 1;
                if bullet.timer_count > bullet.timer_limit {
                    entities.delete(entity).unwrap();
                }
            }

            for &collided in &result.collided {
                match colliders.get(collided).unwrap().tag.as_str() {
                    "Wall" => {
                        bullet.reflect_count += 1;
                        if bullet.reflect_count > bullet.reflect_limit {
                            entities.delete(entity).unwrap();
                        }
                    }
                    "Player" => {
                        if players.get(collided).unwrap().team != bullet.team && !bullet.pierce {
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
