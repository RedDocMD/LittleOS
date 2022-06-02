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
    cpu::wait_forever();
}
