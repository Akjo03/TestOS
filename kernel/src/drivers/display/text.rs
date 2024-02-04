use alloc::borrow::ToOwned;
use alloc::rc::Rc;
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
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
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
    buffer: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    cursor_position: Position,
    font: Option<Rc<RefCell<Fonts>>>,
    screen_size: Option<Size>
} #[allow(dead_code)] impl TextDisplayDriver<'_> {
    pub fn init(&mut self, args: &mut TextDisplayDriverArgs) {
        self.font = Some(args.font.clone());
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

                self.buffer[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: color
                };
                self.cursor_position.x += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str, color: ColorCode) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte, color),
                _ => self.write_byte(0xfe, color)
            }
        }
    }

    pub fn write_line(&mut self, s: &str, color: ColorCode) {
        self.write_string(s, color);
        for _ in self.cursor_position.x..BUFFER_WIDTH {
            self.write_byte(b' ', color);
        }
        self.new_line();
    }

    pub fn clear_row(&mut self, row: usize, color: ColorCode) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: color,
        };
        self.buffer[row] = [blank; BUFFER_WIDTH];
    }

    pub fn new_line(&mut self) {
        if self.cursor_position.x >= BUFFER_WIDTH {
            self.cursor_position.x = 0;
            if self.cursor_position.y < BUFFER_HEIGHT - 1 {
                self.cursor_position.y += 1;
            }
            for row in self.cursor_position.y..BUFFER_HEIGHT {
                self.buffer[row] = [ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(TextColor::Black, TextColor::Black),
                }; BUFFER_WIDTH];
            }
        }
    }
} impl<'a> CommonDisplayDriver<'a> for TextDisplayDriver<'a> {
    fn new() -> Self { Self {
        display: None,
        buffer: [[ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(TextColor::Black, TextColor::Black),
        }; BUFFER_WIDTH]; BUFFER_HEIGHT],
        cursor_position: Position::new(0, 0),
        font: None,
        screen_size: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = self.display.as_mut() {
            if let Some(font) = self.font.as_ref() {
                let mut display = display.borrow_mut();
                let font = font.borrow_mut().to_owned();
                let mut x = 0;
                let mut y = 0;
                for row in self.buffer.iter() {
                    for &screen_char in row.iter() {
                        let text_color: Colors = TextColor::from_u8(
                            screen_char.color_code.0 & 0x0f
                        ).unwrap_or(TextColor::White).into();
                        let text_color: Color = text_color.into();

                        let background_color: Colors = TextColor::from_u8(
                            (screen_char.color_code.0 & 0xf0) >> 4
                        ).unwrap_or(TextColor::Black).into();
                        let background_color: Option<Color> = Some(background_color.into());

                        display.draw_char(
                            screen_char.ascii_character as char, Position::new(x, y),
                            text_color, background_color,
                            font.into(), false, false,
                            TextBaseline::Top, TextAlignment::Left, TextLineHeight::Half
                        );

                        x += font.get_size().width;
                    }
                    x = 0;
                    y += font.get_size().height;
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