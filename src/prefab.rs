use amethyst::{
    assets::{PrefabData, PrefabError, ProgressCounter},
    core::Transform,
    derive::PrefabData,
    ecs::prelude::*,
    renderer::{CameraPrefab, SpriteRender, SpriteSheetHandle, Transparent},
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
    pub shield: Option<Shield>,
    pub spawn_point: Option<SpawnPoint>,
    pub map: Option<Map>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteRenderPrefab {
    pub sprite_number: usize,
}
impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        Option<Read<'a, SpriteSheetHandle>>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transparent>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        (sheet, renders, transparents): &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        transparents.insert(entity, Transparent)?;
        let sprite_render = SpriteRender {
            sprite_sheet: sheet.as_mut().unwrap().clone(),
            sprite_number: self.sprite_number,
        };
        renders.insert(entity, sprite_render).map(|_| ())
    }
}

#[derive(PrefabData, Deserialize, Serialize, Default)]
pub struct MapPrefabData {
    pub transform: Option<Transform>,
    pub collider: Option<RectCollider>,
    pub sprite: Option<SpriteRenderPrefab>,
    pub spawn_point: Option<SpawnPoint>,
    #[serde(skip)]
    pub map: Map,
}

impl MapPrefabData {
    pub fn save(world: &mut World) {
        use amethyst::assets::Prefab;
        use ron::ser::{to_string_pretty, PrettyConfig};
        use std::io::{BufWriter, Write};

        let mut prefab = Prefab::<MapPrefabData>::new();
        world.exec(
            |data: (
                Entities<'_>,
                ReadStorage<'_, Transform>,
                ReadStorage<'_, RectCollider>,
                ReadStorage<'_, SpriteRender>,
                ReadStorage<'_, SpawnPoint>,
                ReadStorage<'_, Map>,
            )| {
                let (entities, transforms, colliders, sprites, spawnpoints, maps) = data;
                for (entity, _) in (&entities, &maps).join() {
                    prefab.add(
                        None,
                        Some(MapPrefabData {
                            transform: transforms.get(entity).cloned(),
                            collider: colliders.get(entity).cloned(),
                            sprite: sprites.get(entity).map(|e| SpriteRenderPrefab {
                                sprite_number: e.sprite_number,
                            }),
                            spawn_point: spawnpoints.get(entity).cloned(),
                            map: Map,
                        }),
                    );
                }
            },
        );
        let s = to_string_pretty(&prefab, PrettyConfig::default()).unwrap();
        println!("{}", s);
        let mut f = BufWriter::new(std::fs::File::create("resources/map.ron").unwrap());
        f.write(s.as_bytes()).unwrap();
    }
}
