use amethyst::{
    assets::{PrefabData, PrefabError, ProgressCounter},
    core::nalgebra::*,
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
    renderer::{SpriteRender, SpriteSheetHandle},
};
use serde_derive::{Deserialize, Serialize};

use crate::components::*;

#[derive(PrefabData, Deserialize, Serialize, Default)]
pub struct MyPrefabData {
    pub transform: Option<Transform>,
    pub rigidbody: Option<Rigidbody>,
    pub sprite: Option<SpriteRenderPrefab>,
    pub collider: Option<RectCollider>,
    pub player: Option<Player>,
    pub playable: Option<Playable>,
    pub ai: Option<AI>,
    pub bullet: Option<Bullet>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteRenderPrefab {
    pub sprite_number: usize,
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
        WriteStorage<'a, RectCollider>,
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

        let collider = RectCollider::new("Wall", width, height);
        colliders.insert(entity, collider).map(|_| ())
    }
}
