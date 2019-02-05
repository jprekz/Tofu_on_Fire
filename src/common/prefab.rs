use amethyst::{
    assets::{Handle, Prefab, PrefabData, PrefabLoader},
    ecs::prelude::{Entities, Read, Resources, System, SystemData, WriteStorage},
    shrev::{EventChannel, ReaderId},
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

#[derive(Default)]
pub struct PrefabDataLoaderSystem<T: 'static> {
    reader: Option<ReaderId<T>>,
}

impl<'a, T> System<'a> for PrefabDataLoaderSystem<T>
where
    T: PrefabData<'a> + Send + Sync + 'static,
{
    type SystemData = (Entities<'a>, Read<'a, EventChannel<T>>, T::SystemData);

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<T>>().register_reader());
    }

    fn run(&mut self, (entities, channel, mut prefab_system_data): Self::SystemData) {
        for prefab_data in channel.read(self.reader.as_mut().unwrap()) {
            let entity = entities.create();
            prefab_data
                .add_to_entity(entity, &mut prefab_system_data, &[entity])
                .expect("Unable to add prefab system data to entity");
        }
    }
}
