use amethyst::{
    assets::{Handle, Prefab, PrefabData, PrefabLoader},
    ecs::prelude::{Entities, WriteStorage},
};
use shred_derive::SystemData;

#[derive(SystemData)]
pub struct RuntimePrefabLoader<'a, T>
where
    T: PrefabData<'a> + Send + Sync + 'static,
{
    loader: PrefabLoader<'a, T>,
    entities: Entities<'a>,
    storage: WriteStorage<'a, Handle<Prefab<T>>>,
}

impl<'a, T> RuntimePrefabLoader<'a, T>
where
    T: PrefabData<'a> + Send + Sync + 'static,
{
    pub fn load_main(&mut self, data: T) {
        let RuntimePrefabLoader {
            loader,
            entities,
            storage,
        } = self;
        let prefab_handle = loader.load_from_data(Prefab::new_main(data), ());
        entities.build_entity().with(prefab_handle, storage).build();
    }

    pub fn load(&mut self, prefab: Prefab<T>) {
        let RuntimePrefabLoader {
            loader,
            entities,
            storage,
        } = self;
        let prefab_handle = loader.load_from_data(prefab, ());
        entities.build_entity().with(prefab_handle, storage).build();
    }
}
