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
    let sp: usize;
    unsafe { core::arch::asm!("mov {x}, sp", x = out(reg) sp) };
    kprintln!("Stack pointer : {:#018X}", sp);
    kprintln!("kernel_main   : {:#018X}", kernel_main as *const () as u64);
    cpu::wait_forever();
}
