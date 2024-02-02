use alloc::rc::Rc;
use core::cell::RefCell;

use bootloader_api::info::FrameBufferInfo;

use crate::api::display::Colors;
use crate::drivers::display::{DisplayDriverManager, DisplayDriverType};
use crate::systems::display::Display;

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

    pub fn set_driver(&mut self, driver: DisplayDriverType<'a>) {
        self.driver_manager.set_driver(driver, Rc::clone(&self.display));
    }

    pub fn get_driver(&mut self) -> &mut DisplayDriverType<'a> {
        &mut self.driver_manager.current_driver
    }

    pub fn clear_screen(&mut self) {
        self.driver_manager.clear(Colors::Black.into())
    }
}