use alloc::string::ToString;
use crate::api::display::Fonts;
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverType};
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
    }

    pub fn tick(&mut self, tick: u64) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, ..) => {
                if (tick % 2) == 0 {
                    driver.write_string(tick.to_string().as_str(), ColorCode::new(TextColor::LightRed, TextColor::Black));
                } else {
                    driver.new_line();
                    driver.write_string(tick.to_string().as_str(), ColorCode::new(TextColor::LightGreen, TextColor::Black));
                }
                driver.draw_all();
                driver.clear_buffer(ColorCode::new(TextColor::Black, TextColor::Black))
            },
            _ => panic!("Unsupported display driver!")
        }

        if tick > 20 {
            panic!("Kernel ticked too many times!")
        }
    }

    pub fn halt(&self) -> ! {
        loop {}
    }
}