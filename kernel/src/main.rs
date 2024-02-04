#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    info::FrameBufferInfo
};
use x86_64::VirtAddr;
use crate::internal::memory::{BootInfoFrameAllocator, SimpleBootInfoFrameAllocator};
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
    if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
        let info = frame_buffer.info().clone();
        let buffer = frame_buffer.buffer_mut();
        initialize_framebuffer(buffer, info);
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
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::new(&boot_info.memory_regions)
    };
    if let Err(_) = internal::allocator::init_main_heap(&mut mapper, &mut frame_allocator) {
        panic!("Heap initialization failed!");
    }
    internal::allocator::init_allocator();

    if let Some(frame_buffer) = get_framebuffer() {
        if let Some(frame_buffer_info) = get_framebuffer_info() {
            let mut display_manager = DisplayManager::new(DisplayType::Buffered, frame_buffer, frame_buffer_info);
            display_manager.set_mode(DisplayMode::Dummy);
            display_manager.clear_screen();

            let mut kernel = Kernel::new(
                display_manager
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
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(frame_buffer) = crate::get_framebuffer() {
        if let Some(frame_buffer_info) = get_framebuffer_info() {
            let mut display_manager = DisplayManager::new(DisplayType::Simple, frame_buffer, frame_buffer_info);
            display_manager.set_mode(DisplayMode::Dummy);
            match display_manager.get_driver() {
                _ => {}
            }
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