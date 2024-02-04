use alloc::string::String;
use alloc::vec;
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::drivers::display::text::{ColorCode, TextColor};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    pub running: bool
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>) -> Self {
        Self { display_manager, running: true }
    }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::Font10x20));
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, ..) => {
                let test = vec!['a'; 80 * 25];
                let string: String = test.into_iter().collect();
                driver.write_string(string.as_str(), ColorCode::new(TextColor::White, TextColor::Black));
            },
            _ => panic!("Unsupported display driver!")
        }
        self.display_manager.draw_all();
    }

    pub fn tick(&mut self, _tick: u64) {
        self.running = false;
    }

    pub fn halt(&self) -> ! {
        loop {}
    }
}