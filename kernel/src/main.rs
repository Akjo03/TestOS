#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    info::FrameBufferInfo
};
use x86_64::VirtAddr;
use crate::drivers::display::DisplayDriverType;
use crate::internal::memory::{BootInfoFrameAllocator, SimpleBootInfoFrameAllocator};
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::kernel::Kernel;
use crate::managers::display::{DisplayManager, DisplayMode, DisplayType};

mod internal;
mod kernel;

mod api;
mod systems;
mod drivers;
mod managers;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 1024 * 1024;
    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    initialize_serial_port();

    if let Some(serial_port) = get_serial_port() {
        if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
            let info = frame_buffer.info().clone();
            let buffer = frame_buffer.buffer_mut();
            initialize_framebuffer(buffer, info);

            serial_port.log(format_args!("Frame buffer initialized with resolution {}x{} at {}bpp.",
                info.width, info.height, info.bytes_per_pixel * 8
            ), SerialLoggingLevel::Info);
        } else { loop {} }

        let physical_memory_offset = boot_info.physical_memory_offset.as_ref()
            .expect("Physical memory offset not found!");
        let phys_mem_offset = VirtAddr::new(*physical_memory_offset);
        let mut mapper = unsafe { internal::memory::init(phys_mem_offset) };

        let mut simple_frame_allocator = unsafe {
            SimpleBootInfoFrameAllocator::new(&boot_info.memory_regions)
        };
        if let Err(_) = internal::allocator::init_initial_heap(&mut mapper, &mut simple_frame_allocator) {
            panic!("Initial heap initialization failed!");
        }

        serial_port.log(format_args!("Initialized initial heap with {} bytes.",
            internal::allocator::INITIAL_HEAP_SIZE
        ), SerialLoggingLevel::Info);

        let mut frame_allocator = unsafe {
            BootInfoFrameAllocator::new(&boot_info.memory_regions)
        };
        if let Err(_) = internal::allocator::init_main_heap(&mut mapper, &mut frame_allocator) {
            panic!("Heap initialization failed!");
        }
        internal::allocator::init_allocator();

        serial_port.log(format_args!("Initialized main heap with {} bytes.",
            internal::allocator::HEAP_SIZE
        ), SerialLoggingLevel::Info);

        if let Some(frame_buffer) = get_framebuffer() {
            if let Some(frame_buffer_info) = get_framebuffer_info() {
                let mut display_manager = DisplayManager::new(DisplayType::Buffered, frame_buffer, frame_buffer_info);
                display_manager.set_mode(DisplayMode::Dummy);
                display_manager.clear_screen();

                serial_port.log(format_args!("Display manager initialized using display mode {} and type {}.",
                    display_manager.get_display_mode(), display_manager.get_display_type()
                ), SerialLoggingLevel::Info);

                let mut kernel = Kernel::new(
                    display_manager,
                    serial_port
                );

                kernel.init();

                let mut tick = 0u64;
                while kernel.running {
                    kernel.tick(tick);
                    tick += 1;
                }

                kernel.halt();
            } else { panic!("Frame buffer info not found!") }
        } else { panic!("Frame buffer not found!") }
    } else { panic!("Serial port not found!") }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(frame_buffer) = get_framebuffer() {
        if let Some(frame_buffer_info) = get_framebuffer_info() {
            let mut display_manager = DisplayManager::new(DisplayType::Simple, frame_buffer, frame_buffer_info);
            display_manager.set_mode(DisplayMode::Dummy);

            match display_manager.get_driver() {
                DisplayDriverType::Dummy(driver) => {
                    let mut message_found = false;

                    if let Some(payload) = info.payload().downcast_ref::<&str>() {
                        driver.draw_panic(payload);
                        message_found = true;
                    } else if let Some(payload) = info.payload().downcast_ref::<String>() {
                        driver.draw_panic(payload.as_str());
                        message_found = true;
                    } else if let Some(message) = info.message() {
                        if let Some(message_str) = message.as_str() {
                            driver.draw_panic(message_str);
                            message_found = true;
                        }
                    }

                    if !message_found {
                        driver.draw_panic("No message provided!");
                    }
                }, _ => {}
            }
        }
    }
    if let Some(serial_port) = get_serial_port() {
        if let Some(payload) = info.payload().downcast_ref::<&str>() {
            serial_port.log(format_args!("{}", payload), SerialLoggingLevel::Panic);
        } else if let Some(payload) = info.payload().downcast_ref::<String>() {
            serial_port.log(format_args!("{}", payload), SerialLoggingLevel::Panic);
        } else if let Some(message) = info.message() {
            if let Some(message_str) = message.as_str() {
                serial_port.log(format_args!("{}", message_str), SerialLoggingLevel::Panic);
            }
        }
    }
    loop {}
}

// ------- Internal Static Access To Framebuffer ---------

static mut FRAMEBUFFER: Option<&'static mut [u8]> = None;
static mut FRAMEBUFFER_INFO: Option<FrameBufferInfo> = None;
static FRAMEBUFFER_INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut SERIAL_PORT: Option<SerialPortLogger> = None;

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

fn initialize_serial_port() {
    unsafe {
        SERIAL_PORT = Some(SerialPortLogger::init());
    }
}

fn get_serial_port() -> Option<&'static mut SerialPortLogger> {
    unsafe { SERIAL_PORT.as_mut() }
}