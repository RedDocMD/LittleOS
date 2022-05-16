#![no_main]
#![no_std]
#![feature(asm_const)]

mod boot;
mod cpu;
mod panic;

unsafe fn kernel_init() -> ! {
    kernel_main();
}

fn kernel_main() -> ! {
    cpu::wait_forever();
}
