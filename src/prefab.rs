use amethyst::{
    assets::{Handle, PrefabData, ProgressCounter},
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
    renderer::{camera::CameraPrefab, SpriteRender, SpriteSheet, Transparent},
    Error,
};
use serde_derive::{Deserialize, Serialize};

use crate::ai::AI;
use crate::components::*;

#[derive(PrefabData, Deserialize, Serialize, Default)]
pub struct MyPrefabData {
    pub transform: Option<Transform>,
    pub rigidbody: Option<Rigidbody>,
    pub camera: Option<CameraPrefab>,
    pub sprite: Option<SpriteRenderPrefab>,
    pub collider: Option<RectCollider>,
    pub player: Option<Player>,
    pub playable: Option<Playable>,
    pub ai: Option<AI>,
    pub bullet: Option<Bullet>,
    pub reticle: Option<Reticle>,
    pub reticle_line: Option<ReticleLine>,
    pub shield: Option<Shield>,
    pub item: Option<Item>,
    pub particle: Option<Particle>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteRenderPrefab {
    pub sprite_number: usize,
}
impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        ReadExpect<'a, Handle<SpriteSheet>>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transparent>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (sheet, renders, transparents): &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        transparents.insert(entity, Transparent)?;
        let sprite_render = SpriteRender {
            sprite_sheet: sheet.clone(),
            sprite_number: self.sprite_number,
        };
        renders.insert(entity, sprite_render)?;
        Ok(())
    }
}

#[derive(PrefabData, Deserialize, Serialize, Default)]
pub struct MapPrefabData {
    pub transform: Option<Transform>,
    pub collider: Option<RectCollider>,
    pub sprite: Option<SpriteRenderPrefab>,
    pub spawn_point: Option<SpawnPoint>,
    pub area: Option<Area>,
    pub area_target: Option<AreaTarget>,
    #[serde(skip)]
    pub map: Map,
}
