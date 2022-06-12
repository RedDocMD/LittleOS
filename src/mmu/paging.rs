use core::mem::{self, MaybeUninit};
use core::ptr;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::mmu::page_size_order;

use super::page_size;

#[inline(always)]
fn addr_mask() -> usize {
    let mut mask: usize = !0;
    const TOP_MASK: usize = 0xFFFF_0000_0000_0000;
    mask &= !TOP_MASK;
    mask &= !((1 << page_size_order()) - 1);
    mask
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TableDescriptor(u64);

impl TableDescriptor {
    pub fn new(table_addr: usize) -> TableDescriptor {
        let mut desc: u64 = 0b11; // 0b11 means table descriptor
        desc |= (table_addr & addr_mask()) as u64;
        TableDescriptor(desc)
    }

    pub fn invalid() -> TableDescriptor {
        TableDescriptor(0)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageDescriptor(u64);

#[repr(u8)]
pub enum AccessPermission {
    PrivilegedReadWrite = 0b00,
    ReadWrite = 0b01,
    PrivilegedReadOnly = 0b10,
    ReadOnly = 0b11,
}

impl PageDescriptor {
    pub fn new(page_addr: usize) -> PageDescriptor {
        let mut desc: u64 = 0b01; // 0b01 means block/page descriptor
        desc |= (page_addr & addr_mask()) as u64;
        PageDescriptor(desc)
    }

    pub fn with_xn(mut self, val: bool) -> PageDescriptor {
        const XN_OFF: usize = 54;
        self.0 &= !(1 << XN_OFF);
        self.0 |= (val as u64) << XN_OFF;
        self
    }

    pub fn with_pxn(mut self, val: bool) -> PageDescriptor {
        const PXN_OFF: usize = 53;
        self.0 &= !(1 << PXN_OFF);
        self.0 |= (val as u64) << PXN_OFF;
        self
    }

    pub fn with_ap(mut self, ap: AccessPermission) -> PageDescriptor {
        const AP_OFF: usize = 6;
        self.0 &= !(0b11 << AP_OFF);
        self.0 |= ((ap as u64) & 0b11) << AP_OFF;
        self
    }

    pub fn invalid() -> PageDescriptor {
        PageDescriptor(0)
    }
}

type PageTable = &'static mut [PageDescriptor];
type HigherTable = &'static mut [TableDescriptor];

const L3_TABLES_COUNT: usize = 256;

pub struct PageTables {
    l0_table: HigherTable,
    l1_table: HigherTable,
    l2_table: HigherTable,
    l3_tables: [PageTable; L3_TABLES_COUNT],
}

impl PageTables {
    pub fn new(l0_addr: usize, l1_addr: usize, l2_addr: usize, l3_addr: usize) -> PageTables {
        const ENTRIES_PER_TABLE: usize = 512;

        let l0_table_ptr =
            ptr::slice_from_raw_parts_mut(l0_addr as *mut TableDescriptor, ENTRIES_PER_TABLE);
        let l0_table = unsafe { &mut *l0_table_ptr };
        fill_with_invalid_table_entries(l0_table);

        let l1_table_ptr =
            ptr::slice_from_raw_parts_mut(l1_addr as *mut TableDescriptor, ENTRIES_PER_TABLE);
        let l1_table = unsafe { &mut *l1_table_ptr };
        fill_with_invalid_table_entries(l1_table);

        let l2_table_ptr =
            ptr::slice_from_raw_parts_mut(l2_addr as *mut TableDescriptor, ENTRIES_PER_TABLE);
        let l2_table = unsafe { &mut *l2_table_ptr };
        fill_with_invalid_table_entries(l2_table);

        let l3_tables = {
            let mut l3_tables: [MaybeUninit<PageTable>; L3_TABLES_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for (i, uninit_l3_table) in l3_tables.iter_mut().enumerate() {
                let l3_table_addr = l3_addr + i * page_size();
                let l3_table_ptr = ptr::slice_from_raw_parts_mut(
                    l3_table_addr as *mut PageDescriptor,
                    ENTRIES_PER_TABLE,
                );
                let l3_table = unsafe { &mut *l3_table_ptr };
                fill_with_invalid_page_entries(l3_table);

                uninit_l3_table.write(l3_table);
            }
            unsafe { mem::transmute(l3_tables) }
        };

        PageTables {
            l0_table,
            l1_table,
            l2_table,
            l3_tables,
        }
    }

    pub fn l0_table(&mut self) -> &mut [TableDescriptor] {
        self.l0_table
    }

    pub fn l1_table(&mut self) -> &mut [TableDescriptor] {
        self.l1_table
    }

    pub fn l2_table(&mut self) -> &mut [TableDescriptor] {
        self.l2_table
    }

    pub fn l3_table(&mut self, idx: usize) -> &mut [PageDescriptor] {
        &mut self.l3_tables[idx]
    }

    pub fn load(&self) {
        use cortex_a::{asm::barrier::*, registers::*};
        const VIRT_BITS: u64 = 48;

        TTBR0_EL1.set(self.l0_table.as_ptr() as u64);

        // Virtual address size is 48 bits
        TCR_EL1.modify(TCR_EL1::T0SZ.val(64 - VIRT_BITS) + TCR_EL1::T1SZ.val(64 - VIRT_BITS));
        // Translation granule is 4K
        TCR_EL1.modify(TCR_EL1::TG0::KiB_4 + TCR_EL1::TG1::KiB_4);
        // TODO: Set caching attributes
        // TODO: Set SMP attributes

        // Set PA range
        TCR_EL1.modify(TCR_EL1::IPS.val(ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange)));
        // Set ASID bits (8 or 16)
        match ID_AA64MMFR0_EL1
            .read_as_enum::<ID_AA64MMFR0_EL1::ASIDBits::Value>(ID_AA64MMFR0_EL1::ASIDBits)
        {
            Some(ID_AA64MMFR0_EL1::ASIDBits::Value::Bits_16) => {
                TCR_EL1.modify(TCR_EL1::AS::ASID16Bits)
            }
            Some(ID_AA64MMFR0_EL1::ASIDBits::Value::Bits_8) => {
                TCR_EL1.modify(TCR_EL1::AS::ASID8Bits)
            }
            None => crate::kprintln!("Failed to retrieve ASID bit count"),
        }

        // Check if we have AF and dirty bit support
        const HA_SHIFT: u64 = 39;
        const HD_SHIFT: u64 = 40;
        let mut tcr_el1_val = TCR_EL1.get();
        match hw_af_db_support() {
            AfAndDbSupport::AfSupported => {
                tcr_el1_val |= 1 << HA_SHIFT;
                TCR_EL1.set(tcr_el1_val);
            }
            AfAndDbSupport::AfAndDbSupported => {
                tcr_el1_val |= 1 << HA_SHIFT;
                tcr_el1_val |= 1 << HD_SHIFT;
                TCR_EL1.set(tcr_el1_val);
            }
            _ => {}
        }

        // Now do an ISB to force these changes to be seen
        unsafe { isb(SY) };

        // Enable MMU with SCTLR_EL1
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable);

        // Now do an ISB to force these changes to be seen
        unsafe { isb(SY) };
    }
}

fn id_aa64mmfr1_el1() -> u64 {
    let id: u64;
    unsafe { core::arch::asm!("mrs {x}, ID_AA64MMFR1_EL1", x = out(reg) id) };
    id
}

enum AfAndDbSupport {
    NotSupported,
    AfSupported,
    AfAndDbSupported,
}

fn hw_af_db_support() -> AfAndDbSupport {
    const HAFDBS_MASK: u64 = 0b1111;
    match id_aa64mmfr1_el1() & HAFDBS_MASK {
        0b0000 => AfAndDbSupport::NotSupported,
        0b0001 => AfAndDbSupport::AfSupported,
        0b0010 => AfAndDbSupport::AfAndDbSupported,
        _ => unreachable!("invalid value for HAFDBS field of ID_AA64MMFF1_EL1"),
    }
}

fn fill_with_invalid_table_entries(table: &mut [TableDescriptor]) {
    for el in table.iter_mut() {
        *el = TableDescriptor::invalid();
    }
}

fn fill_with_invalid_page_entries(table: &mut [PageDescriptor]) {
    for el in table.iter_mut() {
        *el = PageDescriptor::invalid();
    }
}
