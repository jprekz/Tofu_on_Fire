use crate::components::Player;
use amethyst::{
    assets::{AssetStorage, Loader},
    audio::*,
    core::Transform,
    ecs::prelude::*,
};
use shred_derive::SystemData;
use specs_derive::Component;

pub struct Sounds {
    pub shoot: SourceHandle,
}

pub fn initialise_audio(world: &mut World) {
    let source_handle = {
        let loader = world.read_resource::<Loader>();
        loader.load("audio/shot1.wav", WavFormat, (), (), &world.read_resource())
    };
    let sounds = Sounds {
        shoot: source_handle,
    };
    world.add_resource(sounds);
    world.add_resource(());
}

#[derive(Component, Clone, Debug)]
pub struct PlayOnce {
    volume: f32,
}

#[derive(SystemData)]
pub struct AudioPlayer<'a> {
    storage: WriteStorage<'a, PlayOnce>,
}

impl<'a> AudioPlayer<'a> {
    pub fn play_once(&mut self, entity: Entity, volume: f32) {
        self.storage.insert(entity, PlayOnce { volume }).unwrap();
    }
}

pub struct MyAudioSystem;
impl<'s> System<'s> for MyAudioSystem {
    type SystemData = (
        WriteStorage<'s, PlayOnce>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, output::Output>>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Player>,
    );

    fn run(
        &mut self,
        (mut play_once, storage, sounds, output, transforms, players): Self::SystemData,
    ) {
        let player_pos = {
            let camera_transform = (&transforms, &players).join().next().unwrap().0;
            camera_transform.translation().xy()
        };
        for (p, transform) in (&play_once, &transforms).join() {
            let p_pos = transform.translation().xy();
            let dist = (player_pos - p_pos).norm();
            let volume = 1.0 - dist / 200.0;
            if volume <= 0.0 {
                continue;
            }
            if let Some(ref output) = output.as_ref() {
                if let Some(sound) = storage.get(&sounds.shoot) {
                    output.play_once(sound, p.volume * volume);
                }
            }
        }
        play_once.clear();
    }
}
