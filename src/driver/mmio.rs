use core::{marker::PhantomData, ops::Deref};

pub const MMIO_BASE: usize = 0x3F00_0000;

pub struct MMIODerefWrapper<T> {
    start_addr: usize,
    phantom: PhantomData<fn() -> T>,
}

impl<T> MMIODerefWrapper<T> {
    /// Safety: The user must pass the correct start_addr.
    pub const unsafe fn new(start_addr: usize) -> Self {
        Self {
            start_addr,
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for MMIODerefWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: The user must pass the correct start_addr.
        unsafe { &*(self.start_addr as *const _) }
    }
}
