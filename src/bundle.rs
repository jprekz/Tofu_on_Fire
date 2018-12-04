use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
};

use systems::*;
use components::*;

#[derive(Default)]
pub struct GameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GameBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(PlayerSystem, "player_system", &["input_system"]);
        builder.add_barrier();
        builder.add(RigidbodySystem, "rigidbody_system", &[]);
        builder.add_barrier();
        builder.add(CollisionSystem::<Player, Wall>::new(), "collision_system", &[]);
        Ok(())
    }
}
