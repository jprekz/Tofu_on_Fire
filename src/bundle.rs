use amethyst::{
    assets::PrefabLoaderSystem,
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
};

use crate::components::*;
use crate::prefab::*;
use crate::systems::*;

#[derive(Default)]
pub struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[]);
        builder.add(PrefabLoaderSystem::<MapTilePrefab>::default(), "", &[]);
        builder.add(PrefabDataLoaderSystem::<MyPrefabData>::default(), "", &[]);

        builder.add_barrier();

        builder.add(PlayableSystem, "playable_system", &["input_system"]);
        builder.add(AISystem, "ai_system", &[]);
        builder.add(
            PlayerSystem,
            "player_system",
            &["playable_system", "ai_system"],
        );
        builder.add(BulletSystem, "bullet_system", &[]);

        builder.add_barrier();

        builder.add(RigidbodySystem, "rigidbody_system", &[]);
        builder.add(
            CollisionSystem::<Player, Wall>::default(),
            "pw_collision_system",
            &["rigidbody_system"],
        );
        builder.add(
            CollisionSystem::<Bullet, Wall>::default()
                .on_collision(|a, _, _| a.on_collision_wall = true),
            "bw_collision_system",
            &["rigidbody_system"],
        );
        builder.add(
            CollisionSystem::<Player, Bullet>::default()
                .with_filter(|a, b| a.team != b.team)
                .on_collision(|a, b, v| {
                    a.damage += 10;
                    a.knock_back = v;
                    b.on_collision_player = true;
                }),
            "pb_collision_system",
            &["rigidbody_system"],
        );
        Ok(())
    }
}
