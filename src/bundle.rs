use amethyst::{
    assets::PrefabLoaderSystemDesc, core::bundle::SystemBundle, core::SystemDesc,
    ecs::prelude::DispatcherBuilder, Error,
};

use crate::ai::AISystem;
use crate::audio::MyAudioSystem;
use crate::prefab::*;
use crate::systems::*;

use crate::common::pause::Pausable;

#[derive(Default)]
pub struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(
        self,
        world: &mut shred::World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            PrefabLoaderSystemDesc::<MapPrefabData>::default().build(world),
            "",
            &[],
        );
        builder.add(
            PrefabLoaderSystemDesc::<MyPrefabData>::default().build(world),
            "",
            &[],
        );

        builder.add_barrier();

        builder.add(Pausable::new(RigidbodySystem), "rigidbody_system", &[]);
        builder.add(
            CollisionSystem::default()
                .collide("Player", "Wall")
                .collide("Bullet", "Wall")
                .collide("Item", "Wall")
                .collide("Particle", "Wall")
                .trigger("Bullet", "Wall")
                .trigger("Player", "Bullet")
                .trigger("Player", "Item")
                .trigger("Player", "Area"),
            "collision_system",
            &["rigidbody_system"],
        );
        builder.add(
            PlayableSystem::default(),
            "playable_system",
            &["input_system"],
        );
        builder.add(AISystem, "ai_system", &[]);
        builder.add(
            Pausable::new(PlayerControlSystem),
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
        builder.add(
            Pausable::new(BulletSystem),
            "bullet_system",
            &["player_control_system"],
        );
        builder.add(
            CameraSystem::default(),
            "camera_system",
            &["player_control_system"],
        );

        builder.add(ParticleSystem, "particle_system", &[]);
        builder.add(ItemSystem, "item_system", &[]);
        builder.add(Pausable::new(AreaSystem::default()), "area_system", &[]);

        builder.add_barrier();
        builder.add(MyAudioSystem, "my_audio_system", &[]);

        Ok(())
    }
}
