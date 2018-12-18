use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
    assets::PrefabLoaderSystem,
};

use crate::components::*;
use crate::systems::*;
use crate::prefab::*;

#[derive(Default)]
pub struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[]);
        builder.add(PrefabLoaderSystem::<MapTilePrefab>::default(), "", &[]);
        builder.add(PlayerSystem, "player_system", &["input_system"]);
        builder.add(EnemySystem, "enemy_system", &[]);
        builder.add(BulletSystem, "bullet_system", &[]);
        builder.add(
            GeneratorSystem::new(),
            "generator_system",
            &["player_system"],
        );
        builder.add_barrier();
        builder.add(RigidbodySystem, "rigidbody_system", &[]);
        builder.add(
            CollisionSystem::<Player, Wall>::default(),
            "pw_collision_system",
            &["rigidbody_system"],
        );
        builder.add(
            CollisionSystem::<Bullet, Wall>::default(),
            "bw_collision_system",
            &["rigidbody_system"],
        );
        Ok(())
    }
}
