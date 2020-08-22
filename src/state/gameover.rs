use amethyst::{
    core::HiddenPropagate,
    ecs::prelude::*,
    input::{is_key_down, InputHandler, StringBindings},
    prelude::*,
    ui::*,
    winit::VirtualKeyCode,
};

use crate::common::pause::Pause;

#[derive(Default)]
pub struct GameOver {
    pub win: usize,
    pub released: bool,
    pub timer: u64,
}

impl SimpleState for GameOver {
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

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.write_resource::<Pause>().on();

        world.exec(
            |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                let tag = if self.win == 0 { "bluewin" } else { "redwin" };
                if let Some(entity) = finder.find(tag) {
                    hidden.remove(entity);
                }
            },
        );
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        self.timer += 1;

        let pressed_any_key = {
            let input = world.read_resource::<InputHandler<StringBindings>>();

            let shot = input.action_is_down("shot").unwrap_or(false);
            let hold = input.action_is_down("hold").unwrap_or(false);
            shot || hold
        };

        if pressed_any_key && self.released {
            // hide gameover
            world.exec(
                |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                    if let Some(entity) = finder.find("bluewin") {
                        if hidden.insert(entity, HiddenPropagate::new()).is_err() {
                            log::warn!("Failed to insert HiddenPropagate component");
                        }
                    }
                    if let Some(entity) = finder.find("redwin") {
                        if hidden.insert(entity, HiddenPropagate::new()).is_err() {
                            log::warn!("Failed to insert HiddenPropagate component");
                        }
                    }
                },
            );

            return Trans::Pop;
        }

        if !pressed_any_key {
            self.released = true;
        }

        Trans::None
    }
}
