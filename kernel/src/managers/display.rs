use alloc::rc::Rc;
use core::cell::RefCell;

use bootloader_api::info::FrameBufferInfo;

use crate::api::display::{Colors, DisplayApi, Fonts, Size};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverManager, DisplayDriverType, DummyDisplayDriver};
use crate::drivers::display::text::{TextDisplayDriver, TextDisplayDriverArgs};
use crate::systems::display::{BufferedDisplay, SimpleDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayMode {
    Unknown,
    Dummy,
    Text(Fonts)
} impl<'a> DisplayMode {
    fn get_driver(self, info: FrameBufferInfo) -> DisplayDriverType<'a> {
        match self {
            DisplayMode::Unknown => DisplayDriverType::Unknown,
            DisplayMode::Dummy => DisplayDriverType::Dummy(
                DummyDisplayDriver::new()
            ), DisplayMode::Text(font) => DisplayDriverType::Text(
                TextDisplayDriver::new(),
                TextDisplayDriverArgs::new(
                    Rc::new(RefCell::new(font)),
                    Size::new(info.width, info.height
                )
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayType {
    Unknown,
    Simple,
    Buffered
} impl<'a> DisplayType {
    pub fn new(&self, frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Rc<RefCell<dyn DisplayApi + 'a>> {
        match self {
            DisplayType::Unknown => panic!("Unknown display type!"),
            DisplayType::Simple => Rc::new(RefCell::new(
                SimpleDisplay::new(frame_buffer, frame_buffer_info)
            )),
            DisplayType::Buffered => Rc::new(RefCell::new(
                BufferedDisplay::new(frame_buffer, frame_buffer_info)
            ))
        }
    }
}

pub struct DisplayManager<'a> {
    display: Rc<RefCell<dyn DisplayApi + 'a>>,
    display_type: DisplayType,
    driver_manager: DisplayDriverManager<'a>
} #[allow(dead_code)] impl<'a> DisplayManager<'a> {
    pub fn new(display_type: DisplayType, frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        let display = display_type.new(frame_buffer, frame_buffer_info);
        let driver_manager = DisplayDriverManager::new();

        Self { display, display_type, driver_manager }
    }

    pub fn set_mode(&mut self, display_mode: DisplayMode) {
        let driver = display_mode.get_driver(self.display.borrow().get_info());

        match driver {
            DisplayDriverType::Text(..) => {
                let display_type = &self.display_type;
                if display_type != &DisplayType::Buffered {
                    panic!("Text mode is only supported with buffered display!");
                }
            }, _ => {}
        }

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