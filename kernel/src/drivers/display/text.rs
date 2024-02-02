use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ops::Deref;
use crate::api::display::{Color, DisplayApi, Fonts, Position, Size, TextAlignment, TextBaseline, TextLineHeight};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriver};
use crate::systems::display::Display;

pub struct TextDisplayDriverArgs {
    font: Rc<RefCell<Fonts>>,
    display_width: usize,
    display_height: usize
} impl TextDisplayDriverArgs {
    pub fn new(
        font: Rc<RefCell<Fonts>>,
        display_width: usize,
        display_height: usize
    ) -> Self { Self {
        font, display_width, display_height
    } }
}

pub struct TextDisplayDriver<'a> {
    display: Option<Rc<RefCell<Display<'a>>>>,
    text_buffer: Option<Vec<u8>>,
    font: Option<Rc<RefCell<Fonts>>>
} #[allow(dead_code)] impl TextDisplayDriver<'_> {
    pub fn init(&mut self, args: &mut TextDisplayDriverArgs) {
        self.font = Some(args.font.clone());
        let font_size = args.font.borrow().get_size();
        let display_width = args.display_width / font_size.width;
        let display_height = args.display_height / font_size.height;
        self.text_buffer = Some(Vec::with_capacity(display_width * display_height));
    }
} impl CommonDisplayDriver for TextDisplayDriver<'_> {
    fn new() -> Self { Self {
        display: None,
        text_buffer: None,
        font: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = &mut self.display {
            if let Some(text) = self.text_buffer.as_ref() {
                if let Ok(text_str) = core::str::from_utf8(&text) {
                    let font = (*self.font.as_ref().unwrap().borrow().deref()).into();
                    display.borrow_mut().draw_text(
                        text_str, Position::new(0, 0),
                        Color::new(255, 255, 255), None,
                        font, false, false,
                        TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                    );
                } else { panic!("Failed to convert text buffer to string!"); }
            } else { panic!("Trying to draw uninitialized text buffer!"); }
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
} impl<'a> DisplayDriver<'a> for TextDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<Display<'a>>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}