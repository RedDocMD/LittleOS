use self::{mini_uart::MINI_UART, qemu::QEMU_OUTPUT};

pub mod gpio;
pub mod mini_uart;
pub mod mmio;
mod qemu;

pub fn console() -> &'static impl crate::print::Write {
    &MINI_UART
}

pub fn qemu_console() -> &'static impl crate::print::Write {
    &QEMU_OUTPUT
}
