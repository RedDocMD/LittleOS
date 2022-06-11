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
    mmu::{
        layout::{boot_alloc_start, l0_pt_start, l1_pt_start, l2_pt_start, l3_pt_start, pt_end},
        page_size,
        paging::{AccessPermission, PageDescriptor, PageTables},
    },
};

use self::mmu::paging::TableDescriptor;

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
    kprintln!("Code end      : {:#018X}", mmu::layout::code_end());

    if let Some(el) = cpu::current_el() {
        kprintln!("Current execution level is EL{}", el);
    } else {
        kprintln!("Failed to retrieve current execution level");
    }

    let mut page_tables =
        PageTables::new(l0_pt_start(), l1_pt_start(), l2_pt_start(), l3_pt_start());
    setup_identity_map(&mut page_tables);

    let alloc = BitmapAllocator::new(pt_end(), boot_alloc_start());

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

fn setup_identity_map(page_tables: &mut PageTables) {
    // Setup the first 128 MiB as identity mapped.
    page_tables.l0_table()[0] = TableDescriptor::new(page_tables.l1_table().as_ptr() as usize);
    page_tables.l1_table()[0] = TableDescriptor::new(page_tables.l2_table().as_ptr() as usize);
    const L2_ENTRY_COUNT: usize = 64;
    const L2_SPAN: usize = 2 * (1 << 20);
    for i in 0..L2_ENTRY_COUNT {
        page_tables.l2_table()[i] = TableDescriptor::new(page_tables.l3_table(i).as_ptr() as usize);
        let l3_table = page_tables.l3_table(i);
        for (j, l3_entry) in l3_table.iter_mut().enumerate() {
            let addr = i * L2_SPAN + j * page_size();
            let desc = PageDescriptor::new(addr).with_ap(AccessPermission::PrivilegedReadWrite);
            *l3_entry = desc;
        }
    }
}
