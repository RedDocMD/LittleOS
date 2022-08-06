use self::{mini_uart::MINI_UART, qemu::QEMU_OUTPUT, uart::PL011_UART};

pub mod console;
pub mod framebuffer;
pub mod gpio;
pub mod mailbox;
pub mod mini_uart;
pub mod mmio;
pub mod qemu;
pub mod uart;

pub fn serial_console() -> &'static impl crate::print::Write {
    &PL011_UART
}

#[allow(dead_code)]
pub fn qemu_console() -> &'static impl crate::print::Write {
    &QEMU_OUTPUT
}

pub trait DeviceDriver {
    fn init(&self);
}

static DRIVERS: [&'static (dyn DeviceDriver + Sync); 2] = [&MINI_UART, &PL011_UART];

pub fn drivers() -> &'static [&'static (dyn DeviceDriver + Sync)] {
    &DRIVERS
}
