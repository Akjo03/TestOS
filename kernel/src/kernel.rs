use crate::api::display::Fonts;
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverType};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    pub running: bool,
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>) -> Self {
        Self { display_manager, running: true }
    }

    pub fn init(&mut self) {
        self.display_manager.set_driver(DisplayMode::Text(Fonts::Font9x18));
    }

    pub fn tick(&mut self, _tick: u64) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, ..) => {
                driver.draw_all();
            },
            _ => panic!("Unsupported display driver type!")
        }
    }

    pub fn halt(&mut self) -> ! {
        loop {}
    }
}