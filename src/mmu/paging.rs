use core::mem::{self, MaybeUninit};
use core::ptr;

use crate::mmu::{page_size_order, TOTAL_MEMORY};

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

pub struct PageTables {
    l0_table: HigherTable,
    l1_table: HigherTable,
    l2_table: HigherTable,
    l3_tables: &'static mut [PageTable],
}

impl PageTables {
    pub fn new(l0_addr: usize, l1_addr: usize, l2_addr: usize, l3_addr: usize) -> PageTables {
        const ENTRIES_PER_TABLE: usize = 512;
        let l3_tables_count: usize = TOTAL_MEMORY / (ENTRIES_PER_TABLE * page_size());

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

        let l3_tables_ptr =
            ptr::slice_from_raw_parts_mut(l3_addr as *mut MaybeUninit<PageTable>, l3_tables_count);
        let l3_tables = unsafe { &mut *l3_tables_ptr };
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
        let l3_tables = unsafe { mem::transmute(l3_tables) };

        PageTables {
            l0_table,
            l1_table,
            l2_table,
            l3_tables,
        }
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
