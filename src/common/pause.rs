use amethyst::ecs::prelude::*;

#[derive(Default)]
pub struct Pause(bool);

impl Pause {
    pub fn paused(&self) -> bool {
        self.0
    }

    pub fn on(&mut self) {
        self.0 = true;
    }

    pub fn off(&mut self) {
        self.0 = false;
    }
}

pub struct Pausable<T>(T);

impl<T> Pausable<T> {
    pub fn new(inner: T) -> Self {
        Pausable(inner)
    }
}

impl<'s, T> System<'s> for Pausable<T>
where
    T: System<'s>,
    T::SystemData: SystemData<'s>,
{
    type SystemData = (T::SystemData, Read<'s, Pause>);

    fn run(&mut self, (inner, pause): Self::SystemData) {
        if pause.paused() {
            return;
        }
        self.0.run(inner);
    }
}
