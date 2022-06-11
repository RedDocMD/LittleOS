use core::cell::UnsafeCell;

extern "Rust" {
    static __pt_end: UnsafeCell<()>;
    static __code_end: UnsafeCell<()>;
    static __boot_alloc_start: UnsafeCell<()>;
    static __l0_pt_start: UnsafeCell<()>;
    static __l1_pt_start: UnsafeCell<()>;
    static __l2_pt_start: UnsafeCell<()>;
    static __l3_pt_start: UnsafeCell<()>;
}

#[inline(always)]
pub fn pt_end() -> usize {
    unsafe { __pt_end.get() as usize }
}

#[inline(always)]
pub fn code_end() -> usize {
    unsafe { __code_end.get() as usize }
}

#[inline(always)]
pub fn boot_alloc_start() -> usize {
    unsafe { __boot_alloc_start.get() as usize }
}

#[inline(always)]
pub fn l0_pt_start() -> usize {
    unsafe { __l0_pt_start.get() as usize }
}

#[inline(always)]
pub fn l1_pt_start() -> usize {
    unsafe { __l1_pt_start.get() as usize }
}

#[inline(always)]
pub fn l2_pt_start() -> usize {
    unsafe { __l2_pt_start.get() as usize }
}

#[inline(always)]
pub fn l3_pt_start() -> usize {
    unsafe { __l3_pt_start.get() as usize }
}
