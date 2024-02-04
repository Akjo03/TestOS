use crate::managers::display::DisplayManager;

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    pub running: bool
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>) -> Self {
        Self { display_manager, running: true }
    }

    pub fn init(&mut self) {
        match self.display_manager.get_driver() {
            _ => panic!("Unsupported display driver!")
        }
    }

    pub fn tick(&mut self, _tick: u64) {
        self.running = false;
    }

    pub fn halt(&self) -> ! {
        loop {}
    }
}