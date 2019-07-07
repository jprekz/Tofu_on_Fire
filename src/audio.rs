use amethyst::{
    assets::{AssetStorage, Loader},
    audio::*,
    core::Transform,
    ecs::prelude::*,
    renderer::Camera,
};
use shred_derive::SystemData;
use specs_derive::Component;

use std::iter::{repeat, Repeat};

pub struct Sounds {
    pub array: Vec<SourceHandle>,
}

pub struct Music {
    pub music: Repeat<SourceHandle>,
}

pub fn initialise_audio(world: &mut World) {
    let sounds = Sounds {
        array: vec![
            load("audio/shot1.wav", world),
            load("audio/shot2.wav", world),
            load("audio/shot3.wav", world),
            load("audio/damage1.wav", world),
            load("audio/death1.wav", world),
        ],
    };
    let music = repeat(load("audio/bgm.wav", world));
    let music = Music { music };
    world.add_resource(sounds);
    world.add_resource(music);
    world.add_resource(());
}
fn load(name: &str, world: &mut World) -> SourceHandle {
    let loader = world.read_resource::<Loader>();
    loader.load(name, WavFormat, (), (), &world.read_resource())
}

#[derive(Component, Clone, Debug)]
pub struct PlayOnce {
    sound: usize,
    volume: f32,
}

#[derive(SystemData)]
pub struct AudioPlayer<'a> {
    storage: WriteStorage<'a, PlayOnce>,
}

impl<'a> AudioPlayer<'a> {
    pub fn play_once(&mut self, entity: Entity, sound: usize, volume: f32) {
        let result = self.storage.insert(entity, PlayOnce { sound, volume });
        if result.is_err() {
            log::warn!("Failed to insert PlayOnce component");
        }
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
        ReadStorage<'s, Camera>,
    );

    fn run(
        &mut self,
        (mut play_once, storage, sounds, output, transforms, cameras): Self::SystemData,
    ) {
        let player_pos = {
            if let Some((camera_transform, _)) = (&transforms, &cameras).join().next() {
                camera_transform.translation().xy()
            } else {
                return;
            }
        };
        for (p, transform) in (&play_once, &transforms).join() {
            let p_pos = transform.translation().xy();
            let dist = (player_pos - p_pos).norm();
            let volume = 1.0 - dist / 200.0;
            if volume <= 0.0 {
                continue;
            }
            let volume = volume * volume;
            if let Some(ref output) = output.as_ref() {
                if let Some(sound) = storage.get(&sounds.array[p.sound]) {
                    output.play_once(sound, p.volume * volume);
                }
            }
        }
        play_once.clear();
    }
}
