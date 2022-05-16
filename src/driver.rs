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

pub trait DeviceDriver {
    fn init(&self);
}

static DRIVERS: [&'static (dyn DeviceDriver + Sync); 1] = [&MINI_UART];

pub fn drivers() -> &'static [&'static (dyn DeviceDriver + Sync)] {
    &DRIVERS
}
