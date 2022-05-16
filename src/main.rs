#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]

mod boot;
mod cpu;
mod driver;
mod panic;
mod print;
mod sync;

unsafe fn kernel_init() -> ! {
    for driver in driver::drivers() {
        driver.init();
    }
    kernel_main();
}

fn kernel_main() -> ! {
    kprintln!("Hello, from LittleOS!");
    cpu::wait_forever();
}
