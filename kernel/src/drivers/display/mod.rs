use alloc::rc::Rc;
use core::cell::RefCell;

use crate::api::display::{Color, DisplayApi, Size};
use crate::drivers::display::graphics::{GraphicsDisplayDriver, GraphicsDisplayDriverArgs};
use crate::drivers::display::text::{TextDisplayDriver, TextDisplayDriverArgs};
use crate::systems::display::Display;

pub mod text;
pub mod graphics;

pub struct DisplayDriverManager<'a> {
    pub(crate) current_driver: DisplayDriverType<'a>
} #[allow(dead_code)] impl<'a> DisplayDriverManager<'a> {
    pub fn new() -> Self { Self {
        current_driver: DisplayDriverType::Unknown
    } }

    pub fn set_driver(&mut self, driver: DisplayDriverType<'a>, display: Rc<RefCell<Display<'a>>>) {
        match &mut self.current_driver {
            DisplayDriverType::Text(ref mut driver, ..) => {
                driver.deactivate();
            }, DisplayDriverType::Graphics(ref mut driver, ..) => {
                driver.deactivate();
            }, DisplayDriverType::Dummy(ref mut driver) => {
                driver.deactivate();
            }, _ => {}
        }
        self.current_driver = driver;
        match &mut self.current_driver {
            DisplayDriverType::Text(ref mut driver, args) => {
                driver.init(args);
                driver.activate(display);
            }, DisplayDriverType::Graphics(ref mut driver, args) => {
                driver.init(args);
                driver.activate(display);
            }, DisplayDriverType::Dummy(ref mut driver) => {
                driver.activate(display);
            }, _ => {}
        }
    }

    pub fn get_driver(&self) -> &DisplayDriverType<'a> {
        &self.current_driver
    }

    pub fn clear(&mut self, color: Color) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.clear(color);
            }, DisplayDriverType::Text(ref mut driver, ..) => {
                driver.clear(color);
            }, DisplayDriverType::Graphics(ref mut driver, ..) => {
                driver.clear(color);
            }, _ => {}
        }
    }
}

#[allow(dead_code)]
pub enum DisplayDriverType<'a> {
    Unknown,
    Dummy(DummyDisplayDriver<'a>),
    Text(TextDisplayDriver<'a>, TextDisplayDriverArgs),
    Graphics(GraphicsDisplayDriver<'a>, GraphicsDisplayDriverArgs)
}

#[allow(unused_variables)]
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
            display.borrow_mut().clear(color);
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