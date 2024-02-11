use alloc::format;
use crate::api::display::Fonts;
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverType};
use crate::drivers::display::text::ScrollDirection;
use crate::get_serial_port;
use crate::internal::serial::SerialLoggingLevel;
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    pub running: bool,
    current_line: u8
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>) -> Self {
        Self {
            display_manager,
            running: true,
            current_line: 1
        }
    }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::default()));

        if let Some(serial_port) = get_serial_port() {
            serial_port.log(format_args!("Kernel initialized and switched display mode to {}.",
                self.display_manager.get_display_mode()
            ), SerialLoggingLevel::Info);
        }
    }

    pub fn tick(&mut self, _tick: u64) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(ref mut driver, ..) => {
                driver.write_line(format!("Line: {}", self.current_line).as_str());

                if self.current_line >= 25 {
                    let scroll_amount = 3;
                    driver.scroll(scroll_amount, ScrollDirection::Up);
                    self.current_line = 25 - scroll_amount as u8;
                }
                self.current_line += 1;

                driver.draw_all();
            },
            _ => panic!("Unsupported display driver!")
        }
    }

    pub fn halt(&self) -> ! {
        loop {}
    }
}