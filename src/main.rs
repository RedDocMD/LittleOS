#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]

mod alloc;
mod boot;
mod cpu;
mod driver;
mod mmu;
mod panic;
mod print;
mod sync;
mod utils;

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
    kprintln!("Data end      : {:#018X}", mmu::layout::data_end());
    kprintln!("Code end      : {:#018X}", mmu::layout::code_end());
    cpu::wait_forever();
}
