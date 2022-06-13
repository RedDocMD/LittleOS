use core::mem::{self, MaybeUninit};
use core::ptr;

use bitfield::bitfield;
use num_enum::IntoPrimitive;

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

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct TableDescriptor(u64);
    impl Debug;

    pub _, set_ns: 63;
    pub u8, from into AccessPermission, _, set_ap: 62, 61;
    pub _, set_xn: 60;
    pub _, set_pxn: 59;
}

impl TableDescriptor {
    pub fn new(table_addr: usize) -> TableDescriptor {
        let mut desc: u64 = 0b11; // First 1 means table descriptor, second means valid
        desc |= (table_addr & addr_mask()) as u64;
        TableDescriptor(desc)
    }

    pub fn invalid() -> TableDescriptor {
        TableDescriptor(0)
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct PageDescriptor(u64);
    impl Debug;

    pub _, set_xn: 54;
    pub _, set_pxn: 53;
    pub _, set_af: 10;
    pub u8, from into Shareability, _, set_sh: 9, 8;
    pub u8, from into AccessPermission, _, set_ap: 7, 6;
    pub u8, from into MemAttrIdx, _, set_attr_idx: 4, 2;
}

#[derive(IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Shareability {
    None = 0b00,
    Outer = 0b10,
    Inner = 0b11,
}

#[derive(IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum AccessPermission {
    PrivilegedReadWrite = 0b00,
    ReadWrite = 0b01,
    PrivilegedReadOnly = 0b10,
    ReadOnly = 0b11,
}

#[derive(IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum MemAttrIdx {
    Normal = 0,
    Device = 1,
    NonCacheable = 2,
}

impl PageDescriptor {
    pub fn new(page_addr: usize) -> PageDescriptor {
        let mut desc: u64 = 0b11; // First 1 means page, second 1 means valid
        desc |= (page_addr & addr_mask()) as u64;
        PageDescriptor(desc)
    }

    pub fn invalid() -> PageDescriptor {
        PageDescriptor(0)
    }
}

type PageTable = &'static mut [PageDescriptor];
type HigherTable = &'static mut [TableDescriptor];

const L3_TABLES_COUNT: usize = 256;

pub struct PageTables {
    l1_table: HigherTable,
    l2_table: HigherTable,
    l3_tables: [PageTable; L3_TABLES_COUNT],
}

impl PageTables {
    pub fn new(l1_addr: usize, l2_addr: usize, l3_addr: usize) -> PageTables {
        const ENTRIES_PER_TABLE: usize = 512;

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
            l1_table,
            l2_table,
            l3_tables,
        }
    }

    pub fn l1_table_mut(&mut self) -> &mut [TableDescriptor] {
        self.l1_table
    }

    pub fn l2_table_mut(&mut self) -> &mut [TableDescriptor] {
        self.l2_table
    }

    pub fn l3_table_mut(&mut self, idx: usize) -> &mut [PageDescriptor] {
        &mut self.l3_tables[idx]
    }

    pub fn l1_table(&self) -> &[TableDescriptor] {
        self.l1_table
    }

    pub fn l2_table(&self) -> &[TableDescriptor] {
        self.l2_table
    }

    pub fn l3_table(&self, idx: usize) -> &[PageDescriptor] {
        self.l3_tables[idx]
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
