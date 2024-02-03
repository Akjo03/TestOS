use alloc::rc::Rc;
use core::cell::RefCell;

use bootloader_api::info::FrameBufferInfo;

use crate::api::display::{Colors, DisplayApi};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverManager, DisplayDriverType, DummyDisplayDriver};
use crate::systems::display::Display;

#[allow(dead_code)]
pub enum DisplayMode {
    Unknown,
    Dummy
} impl<'a> DisplayMode {
    fn get_driver(self, _info: FrameBufferInfo) -> DisplayDriverType<'a> {
        match self {
            DisplayMode::Unknown => DisplayDriverType::Unknown,
            DisplayMode::Dummy => DisplayDriverType::Dummy(
                DummyDisplayDriver::new()
            )
        }
    }
}

pub struct DisplayManager<'a> {
    display: Rc<RefCell<Display<'a>>>,
    driver_manager: DisplayDriverManager<'a>
} #[allow(dead_code)] impl<'a> DisplayManager<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        let display = Display::new(frame_buffer, frame_buffer_info);
        let driver_manager = DisplayDriverManager::new();
        Self {
            display: Rc::new(RefCell::new(display)),
            driver_manager
        }
    }

    pub fn set_driver(&mut self, display_mode: DisplayMode) {
        let driver = display_mode.get_driver(self.display.borrow().get_info());
        self.driver_manager.set_driver(driver, self.display.clone());
    }

    pub fn get_driver(&mut self) -> &mut DisplayDriverType<'a> {
        &mut self.driver_manager.current_driver
    }

    pub fn clear_screen(&mut self) {
        self.driver_manager.clear(Colors::Black.into())
    }

    pub fn draw_all(&mut self) {
        self.driver_manager.draw_all()
    }
}