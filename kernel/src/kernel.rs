use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    display_manager: DisplayManager<'a>,
    serial_logger: &'a mut SerialPortLogger,
    pub running: bool
} impl<'a> Kernel<'a> {
    pub fn new(display_manager: DisplayManager<'a>, serial_logger: &'a mut SerialPortLogger) -> Self {
        Self {
            display_manager,
            serial_logger,
            running: true
        }
    }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::default()));

        self.serial_logger.log(format_args!("Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()),
            SerialLoggingLevel::Info
        );
    }

    pub fn tick(&mut self, _tick: u64) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(_driver, ..) => {},
            _ => panic!("Unsupported display driver!")
        }

        self.running = false;
    }

    pub fn halt(&mut self) -> ! {
        self.serial_logger.log(format_args!("Kernel is halting."), SerialLoggingLevel::Info);

        loop {}
    }
}