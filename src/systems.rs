use amethyst::{
    core::math::*,
    core::transform::*,
    core::Hidden,
    core::Transform,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
    renderer::{Camera, SpriteRender},
    window::ScreenDimensions,
};
use rand::{distributions::*, prelude::*};

use crate::audio::*;
use crate::common::prefab::*;
use crate::components::*;
use crate::prefab::*;
use crate::resources::{WeaponList, Score};

pub use crate::common::{
    collision2d::{CollisionSystem, RigidbodySystem},
    pause::Pause,
    vector2ext::Vector2Ext,
};

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                log::warn!("{} (L{})", e, line!());
                continue;
            }
        }
    };
}

pub struct PlayableSystem {
    before_mouse: Vector2<f32>,
    use_mouse: bool,
}
impl Default for PlayableSystem {
    fn default() -> Self {
        PlayableSystem {
            before_mouse: Vector2::zeros(),
            use_mouse: false,
        }
    }
}
impl<'s> System<'s> for PlayableSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Playable>,
        WriteStorage<'s, Player>,
    );

    fn run(&mut self, (input, screen, mut playables, mut players): Self::SystemData) {
        let axis_xy_value = |x: &str, y: &str| {
            Some(Vector2::new(
                input.axis_value(x)? as f32,
                input.axis_value(y)? as f32,
            ))
        };

        let move_vec = axis_xy_value("move_x", "move_y").unwrap_or(Vector2::zeros());
        let dpad_vec = axis_xy_value("dpad_x", "dpad_y").unwrap_or(Vector2::zeros());
        let left_vec = axis_xy_value("left_x", "left_y").unwrap_or(Vector2::zeros());
        let move_vec = move_vec + dpad_vec + left_vec;
        let (move_r, move_theta) = move_vec.to_polar();
        let move_vec = Vector2::from_polar(move_r.min(1.0), move_theta);

        let aim_vec = axis_xy_value("right_x", "right_y").unwrap_or(Vector2::zeros());
        let (aim_r, _) = aim_vec.to_polar();
        let mouse_vec = input
            .mouse_position()
            .map(|v| {
                Vector2::new(
                    v.0 as f32 - (screen.width() / 2.0),
                    v.1 as f32 - (screen.height() / 2.0),
                )
            })
            .unwrap_or(Vector2::zeros());

        let move_mouse = self.before_mouse != mouse_vec;
        if move_mouse {
            self.use_mouse = true;
        }

        self.before_mouse = mouse_vec;

        let aim_vec = if aim_r > 0.1 {
            self.use_mouse = false;
            aim_vec
        } else if self.use_mouse {
            mouse_vec
        } else {
            move_vec
        };

        let shot = input.action_is_down("shot").unwrap_or(false);
        let hold = input.action_is_down("hold").unwrap_or(false);

        for (_, player) in (&mut playables, &mut players).join() {
            player.input_move = move_vec;
            if !hold {
                let (aim_r, aim_theta) = aim_vec.to_polar();
                if aim_r >= 0.1 {
                    player.input_aim = Vector2::from_polar(1.0, aim_theta);
                };
            }
            player.input_shot = shot;
        }
    }
}

pub struct PlayerControlSystem;
impl<'s> System<'s> for PlayerControlSystem {
    type SystemData = (
        RuntimePrefabLoader<'s, MyPrefabData>,
        AudioPlayer<'s>,
        ReadExpect<'s, WeaponList>,
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
            let shot = player.input_shot;

            rigidbody.acceleration = move_vec * weapon.move_speed;

            if player.trigger_timer > 0 {
                player.trigger_timer -= 1;
            }
            if shot && player.trigger_timer == 0 {
                let bullet_vel = {
                    let (r, theta) = aim_vec.to_polar();
                    let spread =
                        Uniform::new_inclusive(-weapon.bullet_spread, weapon.bullet_spread)
                            .sample(&mut thread_rng());
                    Vector2::from_polar(r, theta + spread)
                };

                let mut bullet_transform = transform.clone();
                bullet_transform.set_translation_z(-1.0);

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
                        weapon.bullet_damage,
                        weapon.bullet_timer_limit,
                        weapon.bullet_reflect_limit,
                        weapon.bullet_knockback,
                        weapon.bullet_slowing,
                        weapon.bullet_pierce,
                    )),
                    ..Default::default()
                });
                player.trigger_timer = weapon.rate;
                audio.play_once(entity, player.weapon, 0.4);
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
        ReadStorage<'s, ColliderResult>,
    );

    fn run(
        &mut self,
        (mut players, bullets, transforms, mut rigidbodies, results): Self::SystemData,
    ) {
        for (player, transform, rigidbody, result) in
            (&mut players, &transforms, &mut rigidbodies, &results).join()
        {
            for collided in &result.collided {
                match collided.tag.as_str() {
                    "Bullet" => {
                        let bullet = skip_fail!(bullets
                            .get(collided.entity)
                            .ok_or("Failed to get bullet component"));
                        if bullet.team == player.team {
                            continue;
                        }
                        player.hp -= bullet.damage;
                        let b_pos = skip_fail!(transforms
                            .get(collided.entity)
                            .ok_or("Failed to get transform component"))
                        .translation()
                        .xy();
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

pub struct PlayerDeathSystem;
impl<'s> System<'s> for PlayerDeathSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Transform>,
        WriteExpect<'s, ParentHierarchy>,
        RuntimePrefabLoader<'s, MyPrefabData>,
        AudioPlayer<'s>,
    );

    fn run(
        &mut self,
        (entities, players, tramsforms, hierarchy, mut prefab_loader, mut audio): Self::SystemData,
    ) {
        use rand::prelude::*;

        for (entity, player) in (&entities, &players).join() {
            if player.hp > 0.0 {
                continue;
            }
            skip_fail!(entities.delete(entity));
            for entity in hierarchy.all_children_iter(entity) {
                skip_fail!(entities.delete(entity));
            }
            let transform = skip_fail!(tramsforms
                .get(entity)
                .ok_or("Failed to get transform component"))
            .clone();
            for _ in 0..10 {
                prefab_loader.load_main(MyPrefabData {
                    transform: Some(transform.clone()),
                    rigidbody: Some(Rigidbody {
                        velocity: Vector2::from_polar(3.0, random::<f32>() * f32::two_pi()),
                        drag: 0.05,
                        bounciness: 0.8,
                        ..Default::default()
                    }),
                    sprite: Some(SpriteRenderPrefab { sprite_number: 15 }),
                    collider: Some(RectCollider::new("Item", 4.0, 4.0)),
                    item: Some(Item {
                        hp: 10.0,
                        timer: 300,
                    }),
                    ..Default::default()
                });
            }
            let mut transform = transform.clone();
            transform.set_scale(Vector3::new(0.5, 0.2, 1.0));
            for _ in 0..12 {
                prefab_loader.load_main(MyPrefabData {
                    transform: Some(transform.clone()),
                    rigidbody: Some(Rigidbody {
                        velocity: Vector2::from_polar(
                            random::<f32>() * 6.0 + 1.0,
                            random::<f32>() * f32::two_pi(),
                        ),
                        drag: 0.05,
                        bounciness: 0.8,
                        auto_rotate: true,
                        ..Default::default()
                    }),
                    sprite: Some(SpriteRenderPrefab { sprite_number: 15 }),
                    collider: Some(RectCollider::new("Particle", 1.0, 1.0)),
                    particle: Some(Particle { timer: 16 }),
                    ..Default::default()
                });
            }
            audio.play_once(entity, 4, 1.0);
        }
    }
}

pub struct ParticleSystem;
impl<'s> System<'s> for ParticleSystem {
    type SystemData = (Entities<'s>, WriteStorage<'s, Particle>);

    fn run(&mut self, (entities, mut particles): Self::SystemData) {
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.timer -= 1;
            if particle.timer < 0 {
                skip_fail!(entities.delete(entity));
                continue;
            }
        }
    }
}

pub struct ItemSystem;
impl<'s> System<'s> for ItemSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Item>,
        ReadStorage<'s, ColliderResult>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Hidden>,
    );

    fn run(&mut self, (entities, mut items, results, mut players, mut hidden): Self::SystemData) {
        for (entity, mut item, result) in (&entities, &mut items, &results).join() {
            item.timer -= 1;
            if item.timer < 120 {
                if item.timer % 8 <= 4 {
                    let _ = hidden.remove(entity);
                } else {
                    let _ = hidden.insert(entity, Hidden);
                }
            }
            if item.timer < 0 {
                skip_fail!(entities.delete(entity));
                continue;
            }

            let collided_players = result
                .collided
                .iter()
                .filter(|collided| collided.tag == "Player")
                .count();
            if collided_players > 0 {
                for collided in &result.collided {
                    match collided.tag.as_str() {
                        "Player" => {
                            let player = skip_fail!(players
                                .get_mut(collided.entity)
                                .ok_or("Failed to get player component"));
                            player.hp = (player.hp + item.hp).min(100.0);
                        }
                        _ => {}
                    }
                }
                skip_fail!(entities.delete(entity));
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
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
            let aim_vec = player.input_aim * 100.0;
            transform.set_translation_x(aim_vec.x);
            transform.set_translation_y(aim_vec.y);
        }
        for (_, parent, transform) in (&lines, &parents, &mut transforms).join() {
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
            let aim_vec = player.input_aim * 100.0;
            let (l, rad) = aim_vec.to_polar();
            transform.set_rotation_euler(0.0, 0.0, rad);
            transform.set_scale(Vector3::new(l / 100.0, 1.0, 1.0));
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
            let player = skip_fail!(players
                .get(parent.entity)
                .ok_or("Failed to get player component"));
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
        ReadStorage<'s, ColliderResult>,
        ReadStorage<'s, Player>,
        AudioPlayer<'s>,
    );

    fn run(&mut self, (entities, mut bullets, results, players, mut audio): Self::SystemData) {
        for (entity, bullet, result) in (&entities, &mut bullets, &results).join() {
            if bullet.timer_limit != 0 {
                bullet.timer_count += 1;
                if bullet.timer_count > bullet.timer_limit {
                    skip_fail!(entities.delete(entity));
                    continue;
                }
            }

            for collided in &result.collided {
                match collided.tag.as_str() {
                    "Wall" => {
                        bullet.reflect_count += 1;
                        if bullet.reflect_count > bullet.reflect_limit {
                            skip_fail!(entities.delete(entity));
                        }
                    }
                    "Player" => {
                        let player = skip_fail!(players
                            .get(collided.entity)
                            .ok_or("Failed to get player component"));
                        if player.team != bullet.team {
                            if !bullet.pierce {
                                skip_fail!(entities.delete(entity));
                            }
                            audio.play_once(entity, 3, 0.2 + bullet.damage / 25.0);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Default)]
pub struct AreaSystem {
    timer: i32,
}
impl<'s> System<'s> for AreaSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Area>,
        ReadStorage<'s, AreaTarget>,
        ReadStorage<'s, ColliderResult>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
        WriteExpect<'s, Score>,
    );

    fn run(
        &mut self,
        (players, areas, targets, results, mut transforms, mut sprites, mut score): Self::SystemData,
    ) {
        self.timer += 1;

        if self.timer % 2 == 0 {
            for (_, transform) in (&targets, &mut transforms).join() {
                transform.append_rotation_z_axis(f32::pi());
            }
        }

        if self.timer % 60 != 0 {
            return;
        }

        for (_, result, transform, sprite) in
            (&areas, &results, &mut transforms, &mut sprites).join()
        {
            let mut p = 0i32;
            for collided in &result.collided {
                let player = skip_fail!(players
                    .get(collided.entity)
                    .ok_or("Failed to get player component"));
                let team = player.team;
                score.score[team as usize] += 1;
                match team {
                    0 => p += 1,
                    1 => p -= 1,
                    _ => (),
                };
            }
            let position = score.score[0] as i32 - score.score[1] as i32;
            let ratio = position as f32 / 100.0 + 0.5;
            let ratio = if ratio < 0.0 {
                0.0
            } else if ratio > 1.0 {
                1.0
            } else {
                ratio
            };
            let position_x = 352.0 * ratio + 176.0;
            transform.set_translation_x(position_x);
            sprite.sprite_number = match p.cmp(&0) {
                std::cmp::Ordering::Equal => 16,
                std::cmp::Ordering::Less => 18,
                std::cmp::Ordering::Greater => 17,
            }
        }
    }
}

#[derive(Default)]
pub struct CameraSystem {
    target_entity: Option<Entity>,
    timer: i32,
}
impl<'s> System<'s> for CameraSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Playable>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Area>,
        Read<'s, Pause>,
    );

    fn run(
        &mut self,
        (entities, cameras, mut transforms, playables, players, areas, pause): Self::SystemData,
    ) {
        if pause.paused() {
            if let Some((_, transform)) = (&areas, &transforms).join().next() {
                let area_x = transform.translation().x;
                let area_y = transform.translation().y;
                for (transform, _) in (&mut transforms, &cameras).join() {
                    let cam_x = transform.translation().x;
                    let cam_y = transform.translation().y;
                    transform.set_translation_x((area_x + cam_x * 9.0) / 10.0);
                    transform.set_translation_y((area_y + cam_y * 9.0) / 10.0);
                }
            }
            return;
        }

        if self.timer > 0 {
            self.timer -= 1;
            if self.timer == 0 {
                self.target_entity = None;
            }
        }

        let target_entity = {
            if let Some((entity, _)) = (&entities, &playables).join().next() {
                self.timer = 0;
                entity
            } else if let Some(entity) = self.target_entity {
                entity
            } else if self.timer > 0 {
                return;
            } else if let Some((entity, _)) = (&entities, &players).join().next() {
                entity
            } else {
                return;
            }
        };
        self.target_entity = Some(target_entity);

        let target_pos = {
            if let Some(transform) = transforms.get(target_entity) {
                transform.translation().xy()
            } else {
                self.target_entity = None;
                self.timer = 60;
                return;
            }
        };

        for (transform, _) in (&mut transforms, &cameras).join() {
            transform.set_translation_x(target_pos.x);
            transform.set_translation_y(target_pos.y);
        }
    }
}
