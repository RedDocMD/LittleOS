use core::cell::UnsafeCell;

extern "Rust" {
    static __code_end: UnsafeCell<()>;

    static __ttbr0_l1_pt_start: UnsafeCell<()>;
    static __ttbr0_l2_pt_start: UnsafeCell<()>;
    static __ttbr0_l3_pt_start: UnsafeCell<()>;
    static __ttbr0_pt_end: UnsafeCell<()>;

    static __ttbr1_l1_pt_start: UnsafeCell<()>;
    static __ttbr1_l2_pt_start: UnsafeCell<()>;
    static __ttbr1_l3_pt_start: UnsafeCell<()>;
    static __ttbr1_pt_end: UnsafeCell<()>;

    static __boot_alloc_start: UnsafeCell<()>;
}

#[inline(always)]
pub fn ttbr1_pt_end() -> usize {
    unsafe { __ttbr1_pt_end.get() as usize }
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
pub fn ttbr0_l1_pt_start() -> usize {
    unsafe { __ttbr0_l1_pt_start.get() as usize }
}

#[inline(always)]
pub fn ttbr0_l2_pt_start() -> usize {
    unsafe { __ttbr0_l2_pt_start.get() as usize }
}

#[inline(always)]
pub fn ttbr0_l3_pt_start() -> usize {
    unsafe { __ttbr0_l3_pt_start.get() as usize }
}

#[inline(always)]
pub fn ttbr1_l1_pt_start() -> usize {
    unsafe { __ttbr1_l1_pt_start.get() as usize }
}

#[inline(always)]
pub fn ttbr1_l2_pt_start() -> usize {
    unsafe { __ttbr1_l2_pt_start.get() as usize }
}

#[inline(always)]
pub fn ttbr1_l3_pt_start() -> usize {
    unsafe { __ttbr1_l3_pt_start.get() as usize }
}
