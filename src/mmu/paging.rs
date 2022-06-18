use bitfield::bitfield;
use num_enum::IntoPrimitive;

use crate::mmu::PAGE_SIZE_ORDER;

#[inline(always)]
fn addr_mask() -> usize {
    let mut mask: usize = !0;
    const TOP_MASK: usize = 0xFFFF_0000_0000_0000;
    mask &= !TOP_MASK;
    mask &= !((1 << PAGE_SIZE_ORDER) - 1);
    mask
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct TableDescriptor(u64);
    impl Debug;

    pub u8, from into AccessPermission, _, set_ap: 62, 61;
    pub _, set_xn: 60;
    pub _, set_pxn: 59;
    pub _, set_af: 10; // This is fake but needed - table descriptors don't have AF flag!
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

impl From<TableDescriptor> for u64 {
    fn from(td: TableDescriptor) -> Self {
        td.0
    }
}

bitfield! {
    #[derive(Clone, Copy)]
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

impl From<PageDescriptor> for u64 {
    fn from(pd: PageDescriptor) -> Self {
        pd.0
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct BlockDescriptor(u64);
    impl Debug;

    pub _, set_xn: 54;
    pub _, set_pxn: 53;
    pub _, set_af: 10;
    pub u8, from into Shareability, _, set_sh: 9, 8;
    pub u8, from into AccessPermission, _, set_ap: 7, 6;
    pub u8, from into MemAttrIdx, _, set_attr_idx: 4, 2;
}

impl BlockDescriptor {
    pub fn invalid() -> BlockDescriptor {
        BlockDescriptor(0)
    }

    pub fn level2(block_addr: usize) -> BlockDescriptor {
        let mut desc: u64 = 0b01; // First 0 means block, second 1 means valid
        const L2_ADDR_MASK: usize = 0x0000_FFFF_FFE0_0000;
        desc |= (block_addr & L2_ADDR_MASK) as u64;
        BlockDescriptor(desc)
    }
}

impl From<BlockDescriptor> for u64 {
    fn from(bd: BlockDescriptor) -> Self {
        bd.0
    }
}
