use crate::mmu::page_size_order;

#[inline(always)]
fn addr_mask() -> usize {
    let mut mask: usize = !0;
    const TOP_MASK: usize = 0xFFFF_0000_0000_0000;
    mask &= !TOP_MASK;
    mask &= !((1 << page_size_order()) - 1);
    mask
}

#[repr(transparent)]
pub struct TableDescriptor(u64);

impl TableDescriptor {
    pub fn new(table_addr: usize) -> TableDescriptor {
        let mut desc: u64 = 0b11; // 0b11 means table descriptor
        desc |= (table_addr & addr_mask()) as u64;
        TableDescriptor(desc)
    }
}

#[repr(transparent)]
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
}
