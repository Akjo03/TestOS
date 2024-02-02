use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    Drawable,
    geometry::{Dimensions, Point},
    mono_font::{MonoFont, MonoTextStyle},
    Pixel,
    pixelcolor::{Rgb888, RgbColor},
    primitives::Rectangle,
    text::{Text, TextStyle, DecorationColor, renderer::CharacterStyle}
};

use crate::api::display::{Color, DisplayApi, Position, TextAlignment, TextBaseline, TextLineHeight};

pub struct Display<'a> {
    frame: DisplayFrame<'a>
} impl<'a> Display<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self {
            frame: DisplayFrame::new(frame_buffer, frame_buffer_info)
        }
    }
} impl<'a> DisplayApi for Display<'a> {
    fn draw_text(&mut self,
                 text: &str, position: Position,
                 text_color: Color, background_color: Option<Color>,
                 font: MonoFont, underline: bool, strikethrough: bool,
                 baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    ) {
        let mut font_style = MonoTextStyle::new(&font, text_color.into());
        font_style.set_background_color(background_color.map(|color| color.into()));

        if underline { font_style.set_underline_color(DecorationColor::TextColor); }
        if strikethrough { font_style.set_strikethrough_color(DecorationColor::TextColor); }

        let mut text_style = TextStyle::default();
        text_style.baseline = baseline.into();
        text_style.alignment = alignment.into();
        text_style.line_height = line_height.into();

        let text = Text::with_text_style(
            text, Point::new(position.x as i32, position.y as i32), font_style, text_style
        );

        text.draw(&mut self.frame).expect("Failed to draw text!");
    }

    fn clear(&mut self, color: Color) {
        for byte_offset in (0..self.frame.frame_buffer.len()).step_by(self.frame.frame_buffer_info.bytes_per_pixel) {
            set_pixel_in_at(self.frame.frame_buffer_info, &mut self.frame.frame_buffer, byte_offset, color)
        }
    }

    fn draw_all(&mut self, buffer: &[u8]) {
        if buffer.len() != self.frame.frame_buffer.len() {
            panic!("Buffer data does not match the expected size");
        }

        for (i, byte) in buffer.iter().enumerate() {
            self.frame.frame_buffer[i] = *byte;
        }
    }

    fn get_info(&self) -> FrameBufferInfo {
        self.frame.frame_buffer_info
    }
}

struct DisplayFrame<'a> {
    frame_buffer: &'a mut [u8],
    frame_buffer_info: FrameBufferInfo
} impl <'a> DisplayFrame<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self { frame_buffer, frame_buffer_info }
    }
} impl<'a> DrawTarget for DisplayFrame<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {

        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            set_pixel_in(self.frame_buffer_info, &mut self.frame_buffer, Position {
                x: point.x as usize,
                y: point.y as usize
            }, Color {
                red: color.r(),
                green: color.g(),
                blue: color.b()
            });
        }
        Ok(())
    }
} impl<'a> Dimensions for DisplayFrame<'a> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            embedded_graphics::geometry::Size::new(
                self.frame_buffer_info.width as u32,
                self.frame_buffer_info.height as u32
            )
        )
    }
}

fn set_pixel_in(info: FrameBufferInfo, frame_buffer: &mut [u8], position: Position, color: Color) {
    let byte_offset = {
        let line_offset = position.y * info.stride;
        let pixel_offset = line_offset + position.x;
        pixel_offset * info.bytes_per_pixel
    };

    set_pixel_in_at(info, frame_buffer, byte_offset, color);
}

fn set_pixel_in_at(info: FrameBufferInfo, frame_buffer: &mut [u8], byte_offset: usize, color: Color) {
    let pixel_buffer = &mut frame_buffer[byte_offset..byte_offset + info.bytes_per_pixel];

    match info.pixel_format {
        PixelFormat::Rgb => {
            pixel_buffer[0] = color.red;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.blue;
        },
        PixelFormat::Bgr => {
            pixel_buffer[0] = color.blue;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.red;
        },
        PixelFormat::U8 => {
            let gray = color.red / 3 + color.green / 3 + color.blue / 3;
            pixel_buffer[0] = gray;
        },
        other => panic!("Unsupported pixel format: {:?}", other)
    }
}