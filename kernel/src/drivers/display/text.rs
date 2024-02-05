use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use crate::api::display::{Color, Colors, DisplayApi, Fonts, Position, Size, TextAlignment, TextBaseline, TextLineHeight};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriver};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TextColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
} impl TextColor {
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TextColor::Black),
            1 => Some(TextColor::Blue),
            2 => Some(TextColor::Green),
            3 => Some(TextColor::Cyan),
            4 => Some(TextColor::Red),
            5 => Some(TextColor::Magenta),
            6 => Some(TextColor::Brown),
            7 => Some(TextColor::LightGray),
            8 => Some(TextColor::DarkGray),
            9 => Some(TextColor::LightBlue),
            10 => Some(TextColor::LightGreen),
            11 => Some(TextColor::LightCyan),
            12 => Some(TextColor::LightRed),
            13 => Some(TextColor::Pink),
            14 => Some(TextColor::Yellow),
            15 => Some(TextColor::White),
            _ => None
        }
    }
} impl Into<Colors> for TextColor {
    fn into(self) -> Colors {
        match self {
            TextColor::Black => Colors::Black,
            TextColor::Blue => Colors::Navy,
            TextColor::Green => Colors::Green,
            TextColor::Cyan => Colors::Teal,
            TextColor::Red => Colors::Maroon,
            TextColor::Magenta => Colors::Purple,
            TextColor::Brown => Colors::Brown,
            TextColor::LightGray => Colors::Silver,
            TextColor::DarkGray => Colors::Gray,
            TextColor::LightBlue => Colors::Navy,
            TextColor::LightGreen => Colors::Lime,
            TextColor::LightCyan => Colors::Aqua,
            TextColor::LightRed => Colors::Red,
            TextColor::Pink => Colors::Fuchsia,
            TextColor::Yellow => Colors::Yellow,
            TextColor::White => Colors::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8); impl ColorCode {
    pub fn new(foreground: TextColor, background: TextColor) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ScreenChar(u16); impl ScreenChar {
    pub fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        let color_code_u16 = (color_code.0 as u16) << 8;
        ScreenChar(color_code_u16 | ascii_character as u16)
    }

    pub fn ascii_character(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn color_code(&self) -> ColorCode {
        ColorCode((self.0 >> 8) as u8)
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub struct TextDisplayDriverArgs {
    font: Rc<RefCell<Fonts>>,
    screen_size: Size
} #[allow(dead_code)] impl TextDisplayDriverArgs {
    pub fn new(font: Rc<RefCell<Fonts>>, screen_size: Size) -> Self {
        Self { font, screen_size }
    }
}

pub struct TextDisplayDriver<'a> {
    display: Option<Rc<RefCell<dyn DisplayApi + 'a>>>,
    buffer: [ScreenChar; BUFFER_WIDTH * BUFFER_HEIGHT],
    buffer_start: usize,
    dirty_flags: [bool; BUFFER_WIDTH * BUFFER_HEIGHT],
    cursor_position: Position,
    font: Option<Fonts>,
    screen_size: Option<Size>
} #[allow(dead_code)] impl TextDisplayDriver<'_> {
    pub fn init(&mut self, args: &mut TextDisplayDriverArgs) {
        self.font = Some(args.font.borrow().to_owned());
        self.screen_size = Some(args.screen_size);
    }

    pub fn write_byte(&mut self, byte: u8, color: ColorCode) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.cursor_position.x >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.cursor_position.y;
                let col = self.cursor_position.x;
                let index = (self.buffer_start + row * BUFFER_WIDTH + col) % (BUFFER_WIDTH * BUFFER_HEIGHT);

                let current_char = self.buffer[index];
                let byte_changed = current_char.ascii_character() != byte;
                let color_changed = current_char.color_code() != color;

                if byte_changed || color_changed {
                    self.buffer[index] = ScreenChar::new(byte, color);
                    self.dirty_flags[index] = true;
                    self.cursor_position.x += 1;
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str, color: ColorCode) {
        let mut index = self.current_buffer_index();
        for byte in s.bytes() {
            self.buffer[index] = ScreenChar::new(byte, color);
            self.dirty_flags[index] = true;
            self.advance_cursor();
            index = self.current_buffer_index();
        }
    }

    pub fn write_line(&mut self, s: &str, color: ColorCode) {
        self.write_string(s, color);
        let remaining_spaces = BUFFER_WIDTH - self.cursor_position.x;
        let start_index = self.current_buffer_index();
        let blank = ScreenChar::new(b' ', color);
        for i in 0..remaining_spaces {
            self.buffer[start_index + i] = blank;
            self.dirty_flags[start_index + i] = true;
        }
        self.new_line();
    }

    pub fn clear_row(&mut self, row: usize, color: ColorCode) {
        let start_index = row * BUFFER_WIDTH;
        let end_index = start_index + BUFFER_WIDTH;
        let blank = ScreenChar::new(b' ', color);

        for i in start_index..end_index {
            self.buffer[i] = blank;
        }

        if row == self.cursor_position.y {
            self.cursor_position.x = 0;
        }
    }

    pub fn clear_buffer(&mut self, color: ColorCode) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row, color);
        }
        self.cursor_position = Position::new(0, 0);
    }

    pub fn new_line(&mut self) {
        self.cursor_position.x = 0;
        if self.cursor_position.y < BUFFER_HEIGHT - 1 {
            self.cursor_position.y += 1;
        } else {
            self.buffer_start = (self.buffer_start + BUFFER_WIDTH) % (BUFFER_WIDTH * BUFFER_HEIGHT);
            self.clear_row(BUFFER_HEIGHT - 1, ColorCode::new(TextColor::Black, TextColor::Black));
        }
    }

    fn current_buffer_index(&self) -> usize {
        (self.buffer_start + self.cursor_position.y * BUFFER_WIDTH + self.cursor_position.x) % (BUFFER_WIDTH * BUFFER_HEIGHT)
    }

    fn advance_cursor(&mut self) {
        self.cursor_position.x += 1;
        if self.cursor_position.x >= BUFFER_WIDTH {
            self.new_line();
        }
    }
} impl<'a> CommonDisplayDriver<'a> for TextDisplayDriver<'a> {
    fn new() -> Self { Self {
        display: None,
        buffer: [ScreenChar::new(
            b' ',
            ColorCode::new(TextColor::Black, TextColor::Black)
        ); BUFFER_WIDTH * BUFFER_HEIGHT],
        buffer_start: 0,
        dirty_flags: [true; BUFFER_WIDTH * BUFFER_HEIGHT],
        cursor_position: Position::new(0, 0),
        font: None,
        screen_size: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = self.display.as_mut() {
            if let Some(font) = self.font {
                let mut display = display.borrow_mut();

                for row in 0..BUFFER_HEIGHT {
                    let row_start = row * BUFFER_WIDTH;
                    if self.dirty_flags[row_start..row_start + BUFFER_WIDTH].iter().any(|&dirty| dirty) {
                        let mut segment_start = 0;
                        let mut last_color = self.buffer[row_start].color_code();

                        for col in 0..BUFFER_WIDTH {
                            let index = (self.buffer_start + row_start + col) % (BUFFER_WIDTH * BUFFER_HEIGHT);
                            let screen_char = self.buffer[index];

                            if self.dirty_flags[index] {
                                if col > segment_start {
                                    let segment = self.buffer[row_start + segment_start..row_start + col].iter()
                                        .map(|char| char.ascii_character() as char)
                                        .collect::<String>();

                                    let x = segment_start * font.get_size().width;
                                    let y = row * font.get_size().height;

                                    let text_color: Colors = TextColor::from_u8(
                                        last_color.0 & 0x0F
                                    ).unwrap_or(TextColor::White).into();
                                    let text_color: Color = text_color.into();

                                    let background_color: Colors = TextColor::from_u8(
                                        last_color.0 >> 4
                                    ).unwrap_or(TextColor::Black).into();
                                    let background_color: Color = background_color.into();

                                    display.draw_text(
                                        &segment, Position::new(x, y),
                                        text_color, Some(background_color),
                                        font.into(), false, false,
                                        TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                                    );
                                }

                                segment_start = col;
                                last_color = screen_char.color_code();
                            }
                        }

                        let segment = self.buffer[row_start + segment_start..row_start + BUFFER_WIDTH].iter()
                            .map(|char| char.ascii_character() as char)
                            .collect::<String>();

                        let x = segment_start * font.get_size().width;
                        let y = row * font.get_size().height;

                        let text_color: Colors = TextColor::from_u8(
                            last_color.0 & 0x0F
                        ).unwrap_or(TextColor::White).into();
                        let text_color: Color = text_color.into();

                        let background_color: Colors = TextColor::from_u8(
                            last_color.0 >> 4
                        ).unwrap_or(TextColor::Black).into();
                        let background_color: Color = background_color.into();

                        display.draw_text(
                            &segment, Position::new(x, y),
                            text_color, Some(background_color),
                            font.into(), false, false,
                            TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                        );
                    }
                }
                display.swap();
            } else { panic!("No font!"); }
        } else { panic!("No display to draw to!"); }
    }

    fn clear(&mut self, color: Color) {
        if let Some(display) = self.display.as_mut() {
            let mut display = display.borrow_mut();
            display.clear(color);
            display.swap();
        } else { panic!("No display to clear!"); }
    }

    fn get_size(&self) -> Size {
        if let Some(display) = self.display.as_ref() {
            let info = display.borrow().get_info();
            Size::new(info.width, info.height)
        } else { Size::new(0, 0) }
    }
} impl<'a> DisplayDriver<'a> for TextDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<dyn DisplayApi + 'a>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}