use alloc::rc::Rc;
use core::cell::RefCell;
use crate::api::display::{Color, DisplayApi, Size};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriver};
use crate::systems::display::Display;

pub struct TextDisplayDriver<'a> {
    display: Option<Rc<RefCell<Display<'a>>>>,
} impl CommonDisplayDriver for TextDisplayDriver<'_> {
    fn new() -> Self { Self {
        display: None
    } }

    fn clear(&mut self, color: Color) {
        if let Some(display) = &mut self.display {
            display.borrow_mut().clear(color);
        }
    }

    fn get_size(&self) -> Size {
        if let Some(display) = &self.display {
            display.borrow().get_size()
        } else { Size::new(0, 0) }
    }
} impl<'a> DisplayDriver<'a> for TextDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<Display<'a>>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}