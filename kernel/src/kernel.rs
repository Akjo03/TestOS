use crate::managers::display::DisplayManager;

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    pub running: bool,
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>) -> Self {
        Self { display_manager, running: true }
    }

    pub fn init(&mut self) {}

    pub fn tick(&mut self) {
        self.running = false;
    }

    pub fn halt(&mut self) -> ! {
        loop {}
    }
}