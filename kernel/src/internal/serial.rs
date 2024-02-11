use core::fmt;
use core::fmt::Write;

#[allow(dead_code)]
pub enum SerialLoggingLevel {
    Debug,
    Info,
    Warning,
    Error,
    Panic
} impl SerialLoggingLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Panic => "PANIC"
        }
    }
}

pub struct SerialPortLogger {
    port: uart_16550::SerialPort
} impl SerialPortLogger {
    pub unsafe fn init() -> Self {
        let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
        port.init();
        Self { port }
    }

    pub fn log(&mut self, args: fmt::Arguments, level: SerialLoggingLevel) {
        self.port.write_fmt(format_args!("[{}]: {}\n", level.as_str(), args)).unwrap();
    }
} impl Write for SerialPortLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.port.write_str(s)
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.port.write_char(c)
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        self.port.write_fmt(args)
    }
}