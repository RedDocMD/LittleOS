#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]

extern crate alloc as std_alloc;

use std_alloc::vec::Vec;

use crate::{
    kalloc::bitmap_alloc::BitmapAllocator,
    mmu::layout::{boot_alloc_start, data_end},
};

mod boot;
mod cpu;
mod driver;
mod kalloc;
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

    if let Some(el) = cpu::current_el() {
        kprintln!("Current execution level is EL{}", el);
    } else {
        kprintln!("Failed to retrieve current execution level");
    }

    let alloc = BitmapAllocator::new(data_end(), boot_alloc_start());

    kprintln!("Using a Vec ...");
    let mut nums = Vec::new_in(&alloc);
    const NUMS_COUNT: usize = 10;
    nums.reserve(NUMS_COUNT);
    for i in 0..NUMS_COUNT {
        nums.push((i + 1) * 2);
    }
    kprintln!("nums = {:?}", nums);

    let mut floats: Vec<f32, _> = Vec::new_in(&alloc);
    const FLOATS_COUNT: usize = 15;
    floats.reserve(FLOATS_COUNT);

    kprintln!("Bootmem start = {:#018X}", boot_alloc_start());
    kprintln!("nums start =    {:#018X}", nums.as_ptr() as usize);
    kprintln!("floats start =  {:#018X}", floats.as_ptr() as usize);

    cpu::wait_forever();
}
