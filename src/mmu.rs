use core::cell::UnsafeCell;

pub mod layout;
pub mod paging;

#[derive(Debug)]
pub enum MemAmt {
    Byte(usize),
    Kib(usize),
    Mib(usize),
    Gib(usize),
}

impl MemAmt {
    pub const fn byte(amt: usize) -> Self {
        Self::Byte(amt)
    }

    pub const fn kib(amt: usize) -> Self {
        Self::Kib(amt)
    }

    pub const fn mib(amt: usize) -> Self {
        Self::Mib(amt)
    }

    pub const fn gib(amt: usize) -> Self {
        Self::Gib(amt)
    }
}

impl From<usize> for MemAmt {
    fn from(amt: usize) -> Self {
        MemAmt::Byte(amt)
    }
}

impl From<MemAmt> for usize {
    fn from(amt: MemAmt) -> Self {
        match amt {
            MemAmt::Byte(amt) => amt,
            MemAmt::Kib(amt) => amt * 1024,
            MemAmt::Mib(amt) => amt * 1024 * 1024,
            MemAmt::Gib(amt) => amt * 1024 * 1024 * 1024,
        }
    }
}

extern "Rust" {
    static __page_size: UnsafeCell<()>;
    static __page_size_order: UnsafeCell<()>;
}

#[inline(always)]
pub fn page_size() -> usize {
    unsafe { __page_size.get() as usize }
}

#[inline(always)]
pub fn page_size_order() -> usize {
    unsafe { __page_size_order.get() as usize }
}

pub const TOTAL_MEMORY: usize = 512 * (1 << 20);

pub const fn align_down(value: usize, align: usize) -> usize {
    (value) & !(align - 1)
}

pub const fn align_up(value: usize, align: usize) -> usize {
    align_down(value + align - 1, align)
}

pub const fn is_aligned(value: usize, align: usize) -> bool {
    value & (align - 1) == 0
}
