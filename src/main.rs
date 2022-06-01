#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]

use core::alloc::{Allocator, Layout};

use crate::alloc::boot_alloc::BootAllocator;

mod alloc;
mod boot;
mod cpu;
mod driver;
mod mmu;
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
    kprintln!("Data end      : {:#018X}", mmu::layout::data_end());
    kprintln!("Code end      : {:#018X}", mmu::layout::code_end());

    let alloc = BootAllocator::new();
    match alloc.allocate_zeroed(Layout::array::<i32>(30).unwrap()) {
        Ok(arr) => {
            let arr = unsafe { arr.as_ref() };
            kprintln!("Arr is of size {}", arr.len());
        }
        Err(_) => kprintln!("Failed to allocate 30 i32's"),
    }
    cpu::wait_forever();
}
