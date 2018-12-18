use amethyst::{
    assets::{PrefabData, PrefabError, ProgressCounter},
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
    renderer::{SpriteRender, SpriteSheetHandle},
    core::nalgebra::*,
};
use serde_derive::{Deserialize, Serialize};

use crate::components::*;

#[derive(PrefabData, Deserialize, Serialize)]
pub struct MyPrefabData {
    transform: Option<Transform>,
    rigidbody: Option<Rigidbody>,
    sprite: Option<SpriteRenderPrefab>,
    collider_player: Option<RectCollider<Player>>,
    collider_wall: Option<RectCollider<Wall>>,
    player: Option<Player>,
    enemy: Option<Enemy>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteRenderPrefab {
    sprite_number: usize,
}
impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        Option<Read<'a, SpriteSheetHandle>>,
        WriteStorage<'a, SpriteRender>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (sheet, render): &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let sprite_render = SpriteRender {
            sprite_sheet: sheet.as_mut().unwrap().clone(),
            sprite_number: self.sprite_number,
        };
        render.insert(entity, sprite_render).map(|_| ())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MapTilePrefab {
    pos: (i32, i32),
    size: (i32, i32),
}
impl<'a> PrefabData<'a> for MapTilePrefab {
    type SystemData = (
        Option<Read<'a, SpriteSheetHandle>>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, RectCollider<Wall>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (sheet, transforms, renders, colliders): &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let width = self.size.0 as f32 * 32.0;
        let height = self.size.1 as f32 * 32.0;
        let x = (self.pos.0 * 32) as f32 + width / 2.0;
        let y = (self.pos.1 * 32) as f32 + height / 2.0;

        let mut transform = Transform::default();
        transform.set_position(Vector3::new(x, y, 0.0));
        transform.set_scale(width / 32.0, height / 32.0, 1.0);
        transforms.insert(entity, transform)?;

        let sprite_render = SpriteRender {
            sprite_sheet: sheet.as_mut().unwrap().clone(),
            sprite_number: 2,
        };
        renders.insert(entity, sprite_render)?;

        let collider = RectCollider::new(width, height);
        colliders.insert(entity, collider).map(|_| ())
    }
}
