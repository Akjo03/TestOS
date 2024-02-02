use crate::api::display::{Colors, Fonts};
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

        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, ..) => {
                driver.write(b"Welcome to the kernel!", Colors::White.into(), None, true, false);
                driver.draw_all();
            },
            _ => panic!("Unsupported display driver type!")
        }
    }

    pub fn tick(&mut self) {
        self.running = false;
    }

    pub fn halt(&mut self) -> ! {
        loop {}
    }
}