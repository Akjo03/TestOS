use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::internal::serial::SerialLoggingLevel;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
} impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) {
    if let Some(serial_logger) = crate::get_serial_port() {
        serial_logger.log(
            format_args!("BREAKPOINT EXCEPTION:\n{:#?}", stack_frame),
            SerialLoggingLevel::Info
        );
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    if let Some(serial_logger) = crate::get_serial_port() {
        serial_logger.log(
            format_args!("DOUBLE FAULT EXCEPTION:\n{:#?}", stack_frame),
            SerialLoggingLevel::Error
        );
    }
    panic!("DOUBLE FAULT EXCEPTION!");
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) { unsafe {
    if let Some(serial_logger) = crate::get_serial_port() {
        serial_logger.log(
            format_args!("TIMER INTERRUPT"),
            SerialLoggingLevel::Info
        );
    }
    PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
} }