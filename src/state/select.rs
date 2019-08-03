use amethyst::{
    ecs::prelude::*,
    input::is_key_down,
    input::InputHandler,
    prelude::*,
    renderer::*,
    ui::*,
    winit::VirtualKeyCode,
};

use crate::resource::*;
use crate::respawn::*;
use crate::state::*;

use crate::common::pause::Pause;

#[derive(Default)]
pub struct Select {
    selecting: u32,
    released: bool,
    timer: i32,
}

impl SimpleState for Select {
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

        if world.read_resource::<Pause>().paused() {
            return Trans::Pop;
        }
        // hide menu
        world.exec(
            |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                if let Some(entity) = finder.find("menu1") {
                    let _ = hidden.insert(entity, HiddenPropagate);
                }
                if let Some(entity) = finder.find("menu2") {
                    let _ = hidden.insert(entity, HiddenPropagate);
                }
                if let Some(entity) = finder.find("menu3") {
                    let _ = hidden.insert(entity, HiddenPropagate);
                }
            },
        );

        // check gameover
        {
            let score = world.read_resource::<Score>();
            let position = score.score[0] as i32 - score.score[1] as i32;
            let ratio = position as f32 / 100.0 + 0.5;
            if ratio <= 0.0 {
                return Trans::Push(Box::new(GameOver {
                    win: 1,
                    ..Default::default()
                }));
            }
            if ratio >= 1.0 {
                return Trans::Push(Box::new(GameOver {
                    win: 0,
                    ..Default::default()
                }));
            }
        }

        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer < 0 {
            self.timer += 1;
        }

        let (up, down, shot, mouse_vec) = {
            let input = world.read_resource::<InputHandler<String, String>>();
            let screen = world.read_resource::<ScreenDimensions>();

            let move_y = input.axis_value("move_y").unwrap_or(0.0);
            let dpad_y = input.axis_value("dpad_y").unwrap_or(0.0);
            let left_y = input.axis_value("left_y").unwrap_or(0.0);
            let y = move_y + dpad_y + left_y;
            let up = y < -0.1;
            let down = y > 0.1;
            let shot = input.action_is_down("shot").unwrap_or(false);
            let mouse_vec = input.mouse_position().map(|v| {
                (
                    v.0 as f32 / screen.width() * 1280.0,
                    v.1 as f32 / screen.height() * 960.0,
                )
            });
            (up, down, shot, mouse_vec)
        };

        if up && self.timer >= 0 {
            self.selecting = (self.selecting + 2) % 3;
            self.timer = -10;
        }
        if down && self.timer <= 0 {
            self.selecting = (self.selecting + 1) % 3;
            self.timer = 10;
        }
        if let Some((x, y)) = mouse_vec {
            if x >= 50.0 && x < 520.0 {
                if y >= 510.0 && y < 620.0 {
                    self.selecting = 0;
                }
                if y >= 630.0 && y < 740.0 {
                    self.selecting = 1;
                }
                if y >= 750.0 && y < 860.0 {
                    self.selecting = 2;
                }
            }
        }

        world.exec(
            |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                if self.selecting == 0 {
                    if let Some(entity) = finder.find("menu1") {
                        hidden.remove(entity);
                    }
                }
                if self.selecting == 1 {
                    if let Some(entity) = finder.find("menu2") {
                        hidden.remove(entity);
                    }
                }
                if self.selecting == 2 {
                    if let Some(entity) = finder.find("menu3") {
                        hidden.remove(entity);
                    }
                }
            },
        );

        if shot && self.released {
            // spawn player
            let mut rh = world.read_resource::<RespawnHandler>().clone();
            rh.respawn_player(world, self.selecting as usize);
            *world.write_resource::<RespawnHandler>() = rh;

            // hide menu
            world.exec(
                |(finder, mut hidden): (UiFinder<'_>, WriteStorage<'_, HiddenPropagate>)| {
                    if let Some(entity) = finder.find("menu1") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                    if let Some(entity) = finder.find("menu2") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                    if let Some(entity) = finder.find("menu3") {
                        let _ = hidden.insert(entity, HiddenPropagate);
                    }
                },
            );

            return Trans::Push(Box::new(Playing::default()));
        }
        if !shot {
            self.released = true;
        }

        Trans::None
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.released = false;
    }
}
