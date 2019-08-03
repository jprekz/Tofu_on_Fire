use amethyst::{
    ecs::prelude::*,
    input::is_key_down,
    prelude::*,
    winit::VirtualKeyCode,
};

use crate::components::*;
use crate::resource::*;
use crate::state::*;

#[derive(Default)]
pub struct Playing {
    timer: Option<i32>,
}

impl SimpleState for Playing {
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) if is_key_down(&event, VirtualKeyCode::Escape) => Trans::Quit,
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        // check gameover
        let score = world.read_resource::<Score>();
        let position = score.score[0] as i32 - score.score[1] as i32;
        let ratio = position as f32 / 100.0 + 0.5;
        if ratio <= 0.0 {
            return Trans::Switch(Box::new(GameOver {
                win: 1,
                ..Default::default()
            }));
        }
        if ratio >= 1.0 {
            return Trans::Switch(Box::new(GameOver {
                win: 0,
                ..Default::default()
            }));
        }

        if self.timer.is_none() && world.read_storage::<Playable>().join().next().is_none() {
            self.timer = Some(60);
        }

        if let Some(ref mut timer) = self.timer {
            *timer -= 1;
            if *timer < 0 {
                return Trans::Pop;
            }
        }

        Trans::None
    }
}
