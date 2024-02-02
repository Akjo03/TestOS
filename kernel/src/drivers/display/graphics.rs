use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use crate::api::display::{Color, DisplayApi, Size};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriver};
use crate::systems::display::Display;

pub struct GraphicsDisplayDriverArgs {
    display_height: usize,
    stride: usize
} impl GraphicsDisplayDriverArgs {
    pub fn new(
        display_height: usize,
        stride: usize
    ) -> Self { Self {
        display_height, stride
    } }
}

pub struct GraphicsDisplayDriver<'a> {
    display: Option<Rc<RefCell<Display<'a>>>>,
    back_buffer: Option<Vec<u8>>
} impl GraphicsDisplayDriver<'_> {
    pub fn init(&mut self, args: &mut GraphicsDisplayDriverArgs) {
        self.back_buffer = Some(Vec::with_capacity(
            args.display_height * args.stride
        ));
    }
} impl CommonDisplayDriver for GraphicsDisplayDriver<'_> {
    fn new() -> Self { Self {
        display: None,
        back_buffer: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = &mut self.display {
            let mut display = display.borrow_mut();
            if let Some(back_buffer) = &self.back_buffer {
                display.draw_all(back_buffer);
            } else { panic!("Trying to draw uninitialized back buffer!"); }
        } else { panic!("Trying to draw uninitialized display!"); }
    }

    fn clear(&mut self, color: Color) {
        if let Some(display) = &mut self.display {
            display.borrow_mut().clear(color);
        } else { panic!("Trying to clear uninitialized display!"); }
    }

    fn get_size(&self) -> Size {
        if let Some(display) = &self.display {
            let info = display.borrow().get_info();
            Size::new(info.width, info.height)
        } else { Size::new(0, 0) }
    }
} impl<'a> DisplayDriver<'a> for GraphicsDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<Display<'a>>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}