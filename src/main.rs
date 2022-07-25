#![no_main]
#![no_std]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]

extern crate alloc as std_alloc;

use core::mem;

use bitflags::bitflags;
use std_alloc::vec::Vec;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{
    driver::{framebuffer::Framebuffer, mmio::MMIO_BASE},
    fonts::{
        psf::{PsfFont, DEFAULT_PSF_FONT_BYTES},
        Font,
    },
    kalloc::{bitmap_alloc::BitmapAllocator, fixed_buffer_alloc::FixedSliceAlloc},
    mmu::{
        layout::*,
        paging::{
            AccessPermission, BlockDescriptor, MemAttrIdx, PageDescriptor, Shareability,
            TableDescriptor,
        },
        PAGE_SIZE,
    },
};

mod boot;
mod cpu;
mod driver;
mod error;
mod fonts;
mod kalloc;
mod mmu;
mod panic;
mod print;
mod sync;

unsafe fn kernel_init() -> ! {
    let mem_limits = {
        #[repr(align(16))]
        struct MboxArr {
            buf: [u8; 256],
        }
        let mut mbox_arr = MboxArr { buf: [0; 256] };
        let alloc = FixedSliceAlloc::new(&mut mbox_arr.buf);
        get_memory_limits(&alloc).unwrap()
    };

    let lower_table = PageTables::new();
    lower_table.setup_identity_map(&mem_limits);
    load_pagetables(lower_table as *mut PageTables as _, 0);

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

    let alloc = BitmapAllocator::new(boot_alloc_bitmap_start(), boot_alloc_start());

    {
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
    }

    let mut framebuffer = Framebuffer::new(&alloc).unwrap();
    kprintln!("{:?}", framebuffer);

    let psf_font = PsfFont::new(DEFAULT_PSF_FONT_BYTES);
    psf_font.render_str("Hello World!", &mut framebuffer, 0, 20);
    kprintln!("Rendered a string!");

    cpu::wait_forever();
}

const ENTRIES_PER_PAGE: usize = PAGE_SIZE / mem::size_of::<u64>();

#[repr(C)]
struct PageTables {
    l1_table: [u64; ENTRIES_PER_PAGE],
    l2_table: [u64; ENTRIES_PER_PAGE],
    l3_table: [u64; ENTRIES_PER_PAGE],
}

impl PageTables {
    fn new() -> &'static mut PageTables {
        let table = unsafe { &mut *(ttbr0_el1_start() as *mut PageTables) };
        table.l1_table.fill(0);
        table.l2_table.fill(0);
        table.l3_table.fill(0);
        table
    }

    fn setup_identity_map(&mut self, mem_limits: &MemLimits) {
        let mut desc = TableDescriptor::new(self.l2_table.as_ptr() as _);
        desc.set_af(true);
        self.l1_table[0] = desc.into();

        let mut desc = TableDescriptor::new(self.l3_table.as_ptr() as _);
        desc.set_af(true);
        self.l2_table[0] = desc.into();

        // Map L3 table for the first 2 MiB of space.
        for i in 0..ENTRIES_PER_PAGE {
            let addr = i * PAGE_SIZE;
            let mut desc = PageDescriptor::new(addr);
            desc.set_af(true);
            desc.set_sh(Shareability::Inner);
            desc.set_attr_idx(MemAttrIdx::Normal);
            if addr < rpi_phys_binary_load_addr() {
                desc.set_ap(AccessPermission::PrivilegedReadWrite);
                desc.set_xn(true);
                desc.set_pxn(true);
            } else if addr < code_end() {
                desc.set_ap(AccessPermission::PrivilegedReadOnly);
            } else {
                desc.set_ap(AccessPermission::PrivilegedReadWrite);
                desc.set_xn(true);
                desc.set_pxn(true);
            }
            self.l3_table[i] = desc.into();
        }

        // Map rest of memory via 2 MiB blocks
        const BLOCK_SIZE: usize = 2 << 20;
        let sram_end = (mem_limits.arm_base + mem_limits.arm_size) / BLOCK_SIZE;
        let vc_end = (mem_limits.vc_base + mem_limits.vc_size) / BLOCK_SIZE;
        const MMIO_START: usize = MMIO_BASE / BLOCK_SIZE;
        for i in 1..ENTRIES_PER_PAGE {
            let addr = i * BLOCK_SIZE;
            let mut desc = BlockDescriptor::level2(addr);
            desc.set_af(true);
            desc.set_xn(true);
            desc.set_pxn(true);
            if i < sram_end {
                desc.set_sh(Shareability::Inner);
                desc.set_attr_idx(MemAttrIdx::Normal);
                desc.set_ap(AccessPermission::PrivilegedReadWrite);
            } else if i < vc_end || i >= MMIO_START {
                desc.set_sh(Shareability::Outer);
                desc.set_attr_idx(MemAttrIdx::Device);
                desc.set_ap(AccessPermission::PrivilegedReadWrite);
            } else {
                desc = BlockDescriptor::invalid();
            }
            self.l2_table[i] = desc.into();
        }
    }
}

pub fn load_pagetables(lower_table: u64, upper_table: u64) {
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
    TTBR0_EL1.set(lower_table);
    TTBR1_EL1.set(upper_table);

    unsafe { dsb(ISH) };
    unsafe { isb(SY) };

    let mut sctlr_el1 = SctlrEl1::get_reg();
    // Set compulsory bits
    sctlr_el1 |= SctlrEl1::SPAN | SctlrEl1::EIS | SctlrEl1::EOS;
    sctlr_el1 -= SctlrEl1::EE
        | SctlrEl1::E0E
        | SctlrEl1::WXN
        | SctlrEl1::I
        | SctlrEl1::SA0
        | SctlrEl1::SA
        | SctlrEl1::C
        | SctlrEl1::A;
    // Enable paging
    sctlr_el1 |= SctlrEl1::M;

    sctlr_el1.set_reg();

    unsafe { isb(SY) };
}

bitflags! {
    #[repr(transparent)]
    struct SctlrEl1: u64 {
        const EE = 1 << 25;
        const E0E = 1 << 24;
        const SPAN = 1 << 23;
        const EIS = 1 << 22;
        const WXN = 1 << 19;
        const I = 1 << 12;
        const EOS = 1 << 11;
        const SA0 = 1 << 4;
        const SA = 1 << 3;
        const C = 1 << 2;
        const A = 1 << 1;
        const M = 1 << 0;
    }
}

impl From<SctlrEl1> for u64 {
    fn from(reg: SctlrEl1) -> Self {
        unsafe { mem::transmute(reg) }
    }
}

impl SctlrEl1 {
    fn get_reg() -> SctlrEl1 {
        let r: u64;
        unsafe { core::arch::asm!("mrs {x}, SCTLR_EL1", x = out(reg) r) };
        unsafe { SctlrEl1::from_bits_unchecked(r) }
    }

    fn set_reg(&self) {
        unsafe { core::arch::asm!("msr SCTLR_EL1, {x}", x = in(reg) Into::<u64>::into(*self)) };
    }
}
