use amethyst::{
    core::math::*,
    core::transform::*,
    core::Transform,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
    window::ScreenDimensions,
};
use rand::{distributions::*, prelude::*};

use crate::audio::*;
use crate::common::{prefab::*, vector2ext::Vector2Ext};
use crate::components::*;
use crate::prefab::*;
use crate::resources::WeaponList;
use crate::skip_fail;

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
