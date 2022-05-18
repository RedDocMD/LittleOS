pub mod layout;

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

pub const PAGE_SIZE: MemAmt = MemAmt::kib(64);
