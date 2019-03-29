use amethyst::{
    core::nalgebra::*, core::transform::components::Parent, core::Transform, ecs::prelude::*,
    input::InputHandler, renderer::SpriteRender,
};
use rand::{distributions::*, prelude::*};

use crate::audio::*;
use crate::common::prefab::*;
use crate::components::*;
use crate::prefab::*;
use crate::weapon::*;

pub use crate::common::{
    collision2d::{CollisionSystem, RigidbodySystem},
    vector2ext::Vector2Ext,
};

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

pub struct PlayerControlSystem;
impl<'s> System<'s> for PlayerControlSystem {
    type SystemData = (
        RuntimePrefabLoader<'s, MyPrefabData>,
        AudioPlayer<'s>,
        Read<'s, WeaponList>,
        (
            Entities<'s>,
            WriteStorage<'s, Player>,
            ReadStorage<'s, Transform>,
            WriteStorage<'s, Rigidbody>,
        ),
    );

    fn run(&mut self, (mut prefab_loader, mut audio, weapon_list, storages): Self::SystemData) {
        let (entities, mut players, transforms, mut rigidbodies) = storages;

        for (entity, player, transform, rigidbody) in
            (&entities, &mut players, &transforms, &mut rigidbodies).join()
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

                prefab_loader.load_main(MyPrefabData {
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
                        sprite_number: weapon.bullet_sprite + player.team as usize,
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
                audio.play_once(entity, player.weapon, 0.2);
            }
        }
    }
}

pub struct PlayerCollisionSystem;
impl<'s> System<'s> for PlayerCollisionSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Bullet>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
        ReadStorage<'s, RectCollider>,
        ReadStorage<'s, ColliderResult>,
    );

    fn run(&mut self, storages: Self::SystemData) {
        let (mut players, bullets, transforms, mut rigidbodies, colliders, results) = storages;

        for (player, transform, rigidbody, result) in
            (&mut players, &transforms, &mut rigidbodies, &results).join()
        {
            for &collided in &result.collided {
                let bullet = bullets.get(collided).unwrap();
                match colliders.get(collided).unwrap().tag.as_str() {
                    "Bullet" if bullet.team != player.team => {
                        player.hp -= 10.0;
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

pub struct PlayerSpawnSystem;
impl<'s> System<'s> for PlayerSpawnSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, SpawnPoint>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Rigidbody>,
    );

    fn run(
        &mut self,
        (entities, mut players, spawnpoints, mut transforms, mut rigidbodies): Self::SystemData,
    ) {
        for (entity, player) in (&entities, &mut players).join() {
            if player.hp > 0.0 {
                continue;
            }
            player.hp = 100.0;
            rigidbodies.get_mut(entity).unwrap().velocity = Vector2::zeros();
            let point = (&spawnpoints, &transforms)
                .join()
                .filter(|(spawnpoint, _)| spawnpoint.team == player.team)
                .next()
                .map(|(_, transform)| transform.translation().clone());
            if let Some(point) = point {
                let transform = transforms.get_mut(entity).unwrap();
                transform.set_x(point.x);
                transform.set_y(point.y);
                player.weapon = (player.weapon + 1) % 3;
            }
        }
    }
}

pub struct ReticleSystem;
impl<'s> System<'s> for ReticleSystem {
    type SystemData = (
        ReadStorage<'s, Reticle>,
        ReadStorage<'s, ReticleLine>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (reticles, lines, parents, mut transforms, players): Self::SystemData) {
        for (_, parent, transform) in (&reticles, &parents, &mut transforms).join() {
            let player = players.get(parent.entity).unwrap();
            let move_vec = player.input_move;
            let aim_vec = player.input_aim;
            let aim_r = aim_vec.x.hypot(aim_vec.y);
            let v = if aim_r < 0.1 { move_vec } else { aim_vec } * 100.0;
            transform.set_x(v.x);
            transform.set_y(v.y);
        }
        for (_, parent, transform) in (&lines, &parents, &mut transforms).join() {
            let player = players.get(parent.entity).unwrap();
            let move_vec = player.input_move;
            let aim_vec = player.input_aim;
            let aim_r = aim_vec.x.hypot(aim_vec.y);
            let v = if aim_r < 0.1 { move_vec } else { aim_vec } * 100.0;
            let (l, rad) = v.to_polar();
            transform.set_rotation_euler(0.0, 0.0, rad);
            transform.set_scale(l / 100.0, 1.0, 1.0);
        }
    }
}

pub struct ShieldSystem;
impl<'s> System<'s> for ShieldSystem {
    type SystemData = (
        ReadStorage<'s, Shield>,
        WriteStorage<'s, SpriteRender>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (shields, mut renders, parents, players): Self::SystemData) {
        for (_, parent, render) in (&shields, &parents, &mut renders).join() {
            let player = players.get(parent.entity).unwrap();
            let hp = player.hp;
            if hp >= 100.0 {
                render.sprite_number = 10;
            } else if hp > 66.6 {
                render.sprite_number = 11;
            } else if hp > 33.3 {
                render.sprite_number = 12;
            } else {
                render.sprite_number = 13;
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
        ReadStorage<'s, ColliderResult>,
        ReadStorage<'s, Player>,
        AudioPlayer<'s>,
    );

    fn run(
        &mut self,
        (entities, mut bullets, colliders, results, players, mut audio): Self::SystemData,
    ) {
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
                        if players.get(collided).unwrap().team != bullet.team {
                            if !bullet.pierce {
                                entities.delete(entity).unwrap();
                            }
                            audio.play_once(entity, 3, 0.5);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
