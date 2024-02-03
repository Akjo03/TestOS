use alloc::rc::Rc;
use core::cell::RefCell;

use crate::api::display::{Color, DisplayApi, Size};
use crate::systems::display::Display;

pub struct DisplayDriverManager<'a> {
    pub current_driver: DisplayDriverType<'a>
} #[allow(dead_code)] impl<'a> DisplayDriverManager<'a> {
    pub fn new() -> Self { Self {
        current_driver: DisplayDriverType::Unknown
    } }

    pub fn set_driver(&mut self, driver: DisplayDriverType<'a>, display: Rc<RefCell<Display<'a>>>) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.deactivate();
            }, _ => {}
        }
        self.current_driver = driver;
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.activate(display);
            }, _ => {}
        }
    }

    pub fn clear(&mut self, color: Color) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.clear(color);
            }, _ => {}
        }
    }

    pub fn draw_all(&mut self) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.draw_all();
            }, _ => {}
        }
    }

    pub fn get_driver(&self) -> &DisplayDriverType<'a> {
        &self.current_driver
    }
}

#[allow(dead_code)]
pub enum DisplayDriverType<'a> {
    Unknown,
    Dummy(DummyDisplayDriver<'a>)
}

trait DisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<Display<'a>>>);
    fn deactivate(&mut self);
}

pub trait CommonDisplayDriver {
    fn new() -> Self;
    fn draw_all(&mut self);
    fn clear(&mut self, color: Color);
    fn get_size(&self) -> Size;
}

pub struct DummyDisplayDriver<'a> {
    display: Option<Rc<RefCell<Display<'a>>>>,
} impl CommonDisplayDriver for DummyDisplayDriver<'_> {
    fn new() -> Self { Self {
        display: None
    } }

    fn draw_all(&mut self) {}

    fn clear(&mut self, color: Color) {
        if let Some(display) = &mut self.display {
            let mut display = display.borrow_mut();
            display.clear(color);
            display.swap();
        }
    }

    fn get_size(&self) -> Size {
        if let Some(display) = &self.display {
            let info = display.borrow().get_info();
            Size::new(info.width, info.height)
        } else { Size::new(0, 0) }
    }
} impl<'a> DisplayDriver<'a> for DummyDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<Display<'a>>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}