use amethyst::{
    assets::PrefabLoaderSystem,
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
};

use crate::ai::AISystem;
use crate::audio::MyAudioSystem;
use crate::prefab::*;
use crate::systems::*;

#[derive(Default)]
pub struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(PrefabLoaderSystem::<MapPrefabData>::default(), "", &[]);
        builder.add(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[]);

        builder.add_barrier();

        builder.add(RigidbodySystem, "rigidbody_system", &[]);
        builder.add(
            CollisionSystem::default()
                .collide("Player", "Wall")
                .collide("Bullet", "Wall")
                .collide("Item", "Wall")
                .trigger("Bullet", "Wall")
                .trigger("Player", "Bullet")
                .trigger("Player", "Item"),
            "collision_system",
            &["rigidbody_system"],
        );
        builder.add(PlayableSystem, "playable_system", &["input_system"]);
        builder.add(AISystem, "ai_system", &["collision_system"]);
        builder.add(
            PlayerControlSystem,
            "player_control_system",
            &["playable_system", "ai_system"],
        );
        builder.add(
            PlayerCollisionSystem,
            "player_collision_system",
            &["player_control_system"],
        );
        builder.add(
            PlayerDeathSystem,
            "player_death_system",
            &["player_collision_system"],
        );
        builder.add(ShieldSystem, "shield_system", &["player_control_system"]);
        builder.add(ReticleSystem, "reticle_system", &["player_control_system"]);
        builder.add(BulletSystem, "bullet_system", &["player_control_system"]);
        builder.add(
            CameraSystem::default(),
            "camera_system",
            &["player_control_system"],
        );

        builder.add(ItemSystem, "item_system", &[]);

        builder.add_barrier();
        builder.add(MyAudioSystem, "my_audio_system", &[]);

        Ok(())
    }
}
