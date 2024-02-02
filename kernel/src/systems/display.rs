use alloc::vec::Vec;

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget,
    Drawable,
    geometry::{Angle, Dimensions, Point},
    mono_font::{MonoFont, MonoTextStyle},
    Pixel,
    pixelcolor::{Rgb888, RgbColor},
    primitives::{Arc, Ellipse, Line, Polyline, Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Styled, Triangle},
    text::{Text, TextStyle}
};
use embedded_graphics::text::DecorationColor;
use embedded_graphics::text::renderer::CharacterStyle;

use crate::api::display::{Color, DisplayApi, Position, Size, TextAlignment, TextBaseline, TextLineHeight};

pub struct Display<'a> {
    frame: DisplayFrame<'a>
} impl<'a> Display<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self {
            frame: DisplayFrame::new(frame_buffer, frame_buffer_info)
        }
    }

    fn get_ellipse(&mut self, center: Position, radius_x: u32, radius_y: u32, color: Color)
        -> Styled<Ellipse, PrimitiveStyle<Rgb888>> {

        Ellipse::with_center(
            Point::new(center.x as i32, center.y as i32),
            embedded_graphics::geometry::Size::new(radius_x, radius_y)
        ).into_styled(PrimitiveStyleBuilder::new()
            .fill_color(color.into())
            .build()
        )
    }
} impl<'a> DisplayApi for Display<'a> {
    fn draw_pixel(&mut self, position: Position, color: Color) {
        set_pixel_in(self.frame.frame_buffer_info, &mut self.frame.frame_buffer, position, color);
    }

    fn draw_line(&mut self, start: Position, end: Position, color: Color, stroke_width: u32) {
        let line = Line::new(
            Point::new(start.x as i32, start.y as i32),
            Point::new(end.x as i32, end.y as i32),
        ).into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(color.into())
            .stroke_width(stroke_width)
            .build()
        );

        self.frame.draw_iter(line.pixels()).expect("Failed to draw line!")
    }

    fn draw_rectangle(&mut self, position: Position, size: Size, color: Color) {
        let rectangle = Rectangle::new(
            Point::new(position.x as i32, position.y as i32),
            embedded_graphics::geometry::Size::new(size.width as u32, size.height as u32)
        ).into_styled(PrimitiveStyleBuilder::new()
            .fill_color(color.into())
            .build()
        );

        self.frame.draw_iter(rectangle.pixels()).expect("Failed to draw rectangle!");
    }

    fn draw_circle(&mut self, center: Position, radius: u32, color: Color) {
        let circle = self.get_ellipse(center, radius, radius, color);

        self.frame.draw_iter(circle.pixels()).expect("Failed to draw circle!");
    }

    fn draw_ellipse(&mut self, center: Position, radius_x: u32, radius_y: u32, color: Color) {
        let ellipse = self.get_ellipse(center, radius_x, radius_y, color);

        self.frame.draw_iter(ellipse.pixels()).expect("Failed to draw ellipse!");
    }

    fn draw_arc(&mut self, center: Position, diameter: u32, start_angle: f32, angle_sweep: f32, color: Color, stroke_width: u32) {
        let arc = Arc::new(
            Point::new(center.x as i32, center.y as i32),
            diameter,
            Angle::from_degrees(start_angle),
            Angle::from_degrees(angle_sweep)
        ).into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(color.into())
            .stroke_width(stroke_width)
            .build()
        );

        self.frame.draw_iter(arc.pixels()).expect("Failed to draw arc!");
    }

    fn draw_triangle(&mut self, point1: Position, point2: Position, point3: Position, color: Color) {
        let triangle = Triangle::new(
            Point::new(point1.x as i32, point1.y as i32),
            Point::new(point2.x as i32, point2.y as i32),
            Point::new(point3.x as i32, point3.y as i32),
        ).into_styled(PrimitiveStyleBuilder::new()
            .fill_color(color.into())
            .build()
        );

        self.frame.draw_iter(triangle.pixels()).expect("Failed to draw triangle!");
    }

    fn draw_polyline(&mut self, points: &[Position], color: Color, stroke_width: u32) {
        let mut polyline_points = Vec::with_capacity(points.len());
        for point in points {
            polyline_points.push(Point::new(point.x as i32, point.y as i32));
        }

        let polyline = Polyline::new(
            polyline_points.as_slice()
        ).into_styled(PrimitiveStyleBuilder::new()
            .stroke_color(color.into())
            .stroke_width(stroke_width)
            .build()
        );

        self.frame.draw_iter(polyline.pixels()).expect("Failed to draw polyline!");
    }

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

    fn draw_image(&mut self, position: Position, size: Size, image: &[u8]) {
        match self.frame.frame_buffer_info.pixel_format {
            PixelFormat::Rgb => {
                let bytes_per_pixel = 3;

                if image.len() != size.width * size.height * bytes_per_pixel {
                    panic!("Image data does not match the expected size");
                }

                for y in 0..size.height {
                    for x in 0..size.width {
                        let start_index = (y * size.width + x) * bytes_per_pixel;
                        let color = Color {
                            red: image[start_index],
                            green: image[start_index + 1],
                            blue: image[start_index + 2],
                        };
                        let pixel_position = Position {
                            x: position.x + x,
                            y: position.y + y,
                        };

                        self.draw_pixel(pixel_position, color);
                    }
                }
            },
            PixelFormat::Bgr => {
                let bytes_per_pixel = 3;

                if image.len() != size.width * size.height * bytes_per_pixel {
                    panic!("Image data does not match the expected size");
                }

                for y in 0..size.height {
                    for x in 0..size.width {
                        let start_index = (y * size.width + x) * bytes_per_pixel;
                        let color = Color {
                            red: image[start_index + 2],
                            green: image[start_index + 1],
                            blue: image[start_index],
                        };
                        let pixel_position = Position {
                            x: position.x + x,
                            y: position.y + y,
                        };

                        self.draw_pixel(pixel_position, color);
                    }
                }
            },
            PixelFormat::U8 => {
                if image.len() != size.width * size.height {
                    panic!("Image data does not match the expected size");
                }

                for y in 0..size.height {
                    for x in 0..size.width {
                        let gray = image[y * size.width + x];
                        let color = Color {
                            red: gray,
                            green: gray,
                            blue: gray,
                        };
                        let pixel_position = Position {
                            x: position.x + x,
                            y: position.y + y,
                        };

                        self.draw_pixel(pixel_position, color);
                    }
                }
            },
            _ => panic!("Unsupported pixel format: {:?}", self.frame.frame_buffer_info.pixel_format)
        }
    }

    fn clear(&mut self, color: Color) {
        for byte_offset in (0..self.frame.frame_buffer.len()).step_by(self.frame.frame_buffer_info.bytes_per_pixel) {
            set_pixel_in_at(self.frame.frame_buffer_info, &mut self.frame.frame_buffer, byte_offset, color)
        }
    }

    fn get_size(&self) -> Size {
        Size {
            width: self.frame.frame_buffer_info.width,
            height: self.frame.frame_buffer_info.height
        }
    }
}

struct DisplayFrame<'a> {
    frame_buffer: &'a mut [u8],
    frame_buffer_info: FrameBufferInfo
} impl <'a> DisplayFrame<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self { frame_buffer, frame_buffer_info }
    }

    pub fn draw_iter<I>(&mut self, pixels: I)
        -> Result<(), <DisplayFrame as DrawTarget>::Error>
        where I: IntoIterator<Item = Pixel<<DisplayFrame<'a> as DrawTarget>::Color>> {

        self.internal_draw_iter(pixels)
    }

    pub fn internal_draw_iter(&mut self, pixels: impl IntoIterator<Item = Pixel<Rgb888>>)
        -> Result<(), <DisplayFrame as DrawTarget>::Error> {

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
} impl<'a> DrawTarget for DisplayFrame<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {

        self.internal_draw_iter(pixels).expect("Failed to draw!");
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