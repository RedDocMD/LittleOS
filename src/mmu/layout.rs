use core::cell::UnsafeCell;

extern "Rust" {
    static __data_end: UnsafeCell<()>;
    static __code_end: UnsafeCell<()>;
}

#[inline(always)]
pub fn data_end() -> usize {
    unsafe { __data_end.get() as usize }
}

#[inline(always)]
pub fn code_end() -> usize {
    unsafe { __code_end.get() as usize }
}
