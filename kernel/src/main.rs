#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;
use core::ptr::addr_of;
use core::sync::atomic::{AtomicBool, Ordering};

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    draw_target::DrawTarget, Drawable,
    geometry::{Dimensions, Point},
    mono_font::{
        iso_8859_1::FONT_9X18,
        MonoTextStyleBuilder
    },
    Pixel,
    pixelcolor::{Rgb888, RgbColor},
    primitives::Rectangle,
    text::{Baseline, Text, TextStyleBuilder}
};
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::kernel::Kernel;
use crate::managers::display::{DisplayManager, DisplayMode};

mod kernel;
mod api;
mod systems;
mod drivers;
mod managers;

static mut BOOT_REGION: [u8; 1024 * 1024 * 2] = [0; 1024 * 1024 * 2];

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
    ClaimOnOom::new(Span::from_const_array(addr_of!(BOOT_REGION)))
}).lock();

const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.kernel_stack_size = 1024 * 1024;
    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
        let info = frame_buffer.info().clone();
        let buffer = frame_buffer.buffer_mut();
        initialize_framebuffer(buffer, info);

        let mut display_manager = DisplayManager::new(
            get_framebuffer().unwrap(),
            get_framebuffer_info().unwrap()
        );
        display_manager.set_driver(DisplayMode::Dummy);
        display_manager.clear_screen();

        let mut kernel = Kernel::new(
            display_manager
        );

        kernel.init();
        while kernel.running { kernel.tick(); }

        kernel.halt()
    } else { loop {} }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let display = get_framebuffer().map(|fb| KernelFramebufferWrapper::new(fb, get_framebuffer_info().unwrap()));
    let mut message_found = false;
    if let Some(mut display) = display {
        display.clear(Rgb888::new(0, 0, 255));
        display.draw_text("Kernel Panic -- please reboot your machine! See message below:", Point::new(0, 0), Rgb888::new(255, 255, 255), Rgb888::new(0, 0, 255));
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            display.draw_text(payload, Point::new(0, 18), Rgb888::new(255, 255, 255), Rgb888::new(0, 0, 255));
            message_found = true;
        } else if let Some(payload) = info.payload().downcast_ref::<String>() {
            display.draw_text(&payload, Point::new(0, 18), Rgb888::new(255, 255, 255), Rgb888::new(0, 0, 255));
            message_found = true;
        } else if let Some(message) = info.message() {
            if let Some(message_str) = message.as_str() {
                display.draw_text(message_str, Point::new(0, 18), Rgb888::new(255, 255, 255), Rgb888::new(0, 0, 255));
                message_found = true;
            }
        }

        if !message_found {
            display.draw_text("No message provided.", Point::new(0, 18), Rgb888::new(255, 255, 255), Rgb888::new(0, 0, 255));
        }
    }
    loop {}
}

// ------- Internal Static Access To Framebuffer ---------

static mut FRAMEBUFFER: Option<&'static mut [u8]> = None;
static mut FRAMEBUFFER_INFO: Option<FrameBufferInfo> = None;
static FRAMEBUFFER_INITIALIZED: AtomicBool = AtomicBool::new(false);

fn initialize_framebuffer(fb: &'static mut [u8], info: FrameBufferInfo) {
    unsafe {
        FRAMEBUFFER = Some(fb);
        FRAMEBUFFER_INFO = Some(info);
    }
    FRAMEBUFFER_INITIALIZED.store(true, Ordering::SeqCst);
}

fn get_framebuffer() -> Option<&'static mut [u8]> {
    if FRAMEBUFFER_INITIALIZED.load(Ordering::SeqCst) {
        unsafe { FRAMEBUFFER.as_mut().map(|fb| &mut **fb) }
    } else {
        None
    }
}

fn get_framebuffer_info() -> Option<FrameBufferInfo> {
    if FRAMEBUFFER_INITIALIZED.load(Ordering::SeqCst) {
        unsafe { FRAMEBUFFER_INFO.clone() }
    } else {
        None
    }
}

struct KernelFramebuffer<'a> {
    frame_buffer: &'a mut [u8],
    frame_buffer_info: FrameBufferInfo
} impl<'a> DrawTarget for KernelFramebuffer<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {

        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            set_pixel_in(self.frame_buffer_info, &mut self.frame_buffer, point, color);
        }
        Ok(())
    }
} impl<'a> Dimensions for KernelFramebuffer<'a> {
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

struct KernelFramebufferWrapper<'a> {
    display: KernelFramebuffer<'a>
} impl<'a> KernelFramebufferWrapper<'a> {
    fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self { display: KernelFramebuffer { frame_buffer, frame_buffer_info } }
    }

    fn draw_text(
        &mut self,
        text: &str,
        point: Point,
        text_color: Rgb888,
        background_color: Rgb888
    ) {
        let font_style = MonoTextStyleBuilder::new()
            .font(&FONT_9X18)
            .text_color(text_color)
            .background_color(background_color)
            .build();

        let text = Text::with_text_style(
            text,
            Point::new(point.x, point.y),
            font_style,
            TextStyleBuilder::new()
                .baseline(Baseline::Top)
                .build()
        );

        if let Ok(_) = text.draw(&mut self.display) {}
    }

    fn clear(&mut self, color: Rgb888) {
        for byte_offset in (0..self.display.frame_buffer.len()).step_by(self.display.frame_buffer_info.bytes_per_pixel) {
            set_pixel_in_at(self.display.frame_buffer_info, &mut self.display.frame_buffer, byte_offset, color)
        }
    }
}

fn set_pixel_in(info: FrameBufferInfo, frame_buffer: &mut [u8], point: Point, color: Rgb888) {
    let byte_offset = {
        let line_offset = point.y * info.stride as i32;
        let pixel_offset = line_offset + point.x;
        pixel_offset * info.bytes_per_pixel as i32
    };

    set_pixel_in_at(info, frame_buffer, byte_offset as usize, color);
}

fn set_pixel_in_at(info: FrameBufferInfo, frame_buffer: &mut [u8], byte_offset: usize, color: Rgb888) {
    let pixel_buffer = &mut frame_buffer[byte_offset..byte_offset + info.bytes_per_pixel];

    match info.pixel_format {
        PixelFormat::Rgb => {
            pixel_buffer[0] = color.r();
            pixel_buffer[1] = color.g();
            pixel_buffer[2] = color.b();
        },
        PixelFormat::Bgr => {
            pixel_buffer[0] = color.b();
            pixel_buffer[1] = color.g();
            pixel_buffer[2] = color.r();
        },
        PixelFormat::U8 => {
            let gray = color.r() / 3 + color.g() / 3 + color.b() / 3;
            pixel_buffer[0] = gray;
        },
        _ => {}
    }
}