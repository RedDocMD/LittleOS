#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]

extern crate alloc as std_alloc;

use std_alloc::vec::Vec;

use crate::{
    kalloc::{boot_alloc::BootAllocator, Allocator, Layout},
    mmu::layout::boot_alloc_start,
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

    let alloc = BootAllocator::new();

    let arr_layout = Layout::array::<i32>(30).unwrap();
    kprintln!("Allocating array ...");
    match alloc.allocate_zeroed(arr_layout) {
        Ok(arr) => {
            let arr =
                unsafe { &mut *core::ptr::slice_from_raw_parts_mut(arr.as_ptr() as *mut i32, 30) };
            kprintln!("Arr is of size {}", arr.len());
            for i in (0..arr.len()).rev() {
                arr[arr.len() - i - 1] = (i * i) as i32;
            }
            kprintln!("Array before sorting: {:?}", arr);
            arr.sort_unstable();
            kprintln!("Array after sorting: {:?}", arr);
            let arr_ptr = core::ptr::NonNull::new(arr.as_mut_ptr() as *mut u8).unwrap();
            unsafe { alloc.deallocate(arr_ptr, arr_layout) };
            kprintln!("Deallocated array ...");
        }
        Err(_) => kprintln!("Failed to allocate 30 i32's"),
    }

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
