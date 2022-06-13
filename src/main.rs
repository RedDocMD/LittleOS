#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]

extern crate alloc as std_alloc;

use bitfield::bitfield;
use std_alloc::vec::Vec;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{
    kalloc::bitmap_alloc::BitmapAllocator,
    mmu::{
        layout::*,
        page_size,
        paging::{AccessPermission, PageDescriptor, PageTables, Shareability, TableDescriptor},
    },
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

    let mut lower_table = PageTables::new(
        ttbr0_l1_pt_start(),
        ttbr0_l2_pt_start(),
        ttbr0_l3_pt_start(),
    );
    setup_identity_map(&mut lower_table);
    let upper_table = PageTables::new(
        ttbr1_l1_pt_start(),
        ttbr1_l2_pt_start(),
        ttbr1_l3_pt_start(),
    );
    load_pagetables(&lower_table, &upper_table);

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

    let alloc = BitmapAllocator::new(ttbr1_pt_end(), boot_alloc_start());

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
    page_tables.l1_table_mut()[0] = TableDescriptor::new(page_tables.l2_table().as_ptr() as usize);
    const L2_ENTRY_COUNT: usize = 64;
    const L2_SPAN: usize = 2 * (1 << 20);
    for i in 0..L2_ENTRY_COUNT {
        page_tables.l2_table_mut()[i] =
            TableDescriptor::new(page_tables.l3_table(i).as_ptr() as usize);
        let l3_table = page_tables.l3_table_mut(i);
        for (j, l3_entry) in l3_table.iter_mut().enumerate() {
            let addr = i * L2_SPAN + j * page_size();
            let mut desc = PageDescriptor::new(addr);
            desc.set_af(true);
            desc.set_ap(AccessPermission::ReadWrite);
            desc.set_sh(Shareability::Inner);
            *l3_entry = desc;
        }
    }
}

pub fn load_pagetables(lower_table: &PageTables, upper_table: &PageTables) {
    use cortex_a::{asm::barrier::*, registers::*};

    // Setup MAIR
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_WriteAlloc
            + MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_WriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_EarlyWriteAck
            + MAIR_EL1::Attr2_Normal_Inner::NonCacheable
            + MAIR_EL1::Attr2_Normal_Outer::NonCacheable,
    );

    // Setup TCR
    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
            + TCR_EL1::TBI1::Ignored
            + TCR_EL1::IPS.val(ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange))
            + TCR_EL1::TG1::KiB_4
            + TCR_EL1::SH1::Inner
            + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD1::EnableTTBR1Walks
            + TCR_EL1::T1SZ.val(25)
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(25),
    );

    unsafe { isb(SY) };

    // Set TTBRx_EL1
    const TTBR_CNP: u64 = 1;
    TTBR0_EL1.set(lower_table.l1_table().as_ptr() as u64 + TTBR_CNP);
    TTBR1_EL1.set(upper_table.l1_table().as_ptr() as u64 + TTBR_CNP);

    unsafe { dsb(ISH) };
    unsafe { isb(SY) };

    let mut sctlr_el1 = SctlrEl1(SCTLR_EL1.get());

    sctlr_el1.set_eis(true);
    sctlr_el1.set_span(true);
    sctlr_el1.set_enrctx(true);

    sctlr_el1.set_ee(false);
    sctlr_el1.set_e0e(false);
    sctlr_el1.set_wxn(false);
    sctlr_el1.set_i(false);
    sctlr_el1.set_sa0(false);
    sctlr_el1.set_sa(false);
    sctlr_el1.set_sa(false);
    sctlr_el1.set_c(false);
    sctlr_el1.set_a(false);

    sctlr_el1.set_m(true);

    SCTLR_EL1.set(sctlr_el1.0);
    unsafe { isb(SY) };
}

bitfield! {
    pub struct SctlrEl1(u64);

    _, set_ee: 25;
    _, set_e0e: 24;
    _, set_span: 23;
    _, set_eis: 22;
    _, set_wxn: 19;
    _, set_i: 12;
    _, set_enrctx: 10;
    _, set_sa0: 4;
    _, set_sa: 3;
    _, set_c: 2;
    _, set_a: 1;
    _, set_m: 0;
}
