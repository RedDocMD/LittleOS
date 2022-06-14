use core::cell::UnsafeCell;

extern "Rust" {
    static __code_end: UnsafeCell<()>;
    static __boot_alloc_start: UnsafeCell<()>;
    static __boot_alloc_bitmap_start: UnsafeCell<()>;
    static __ttbr0_el1_start: UnsafeCell<()>;
    static __rpi_phys_binary_load_addr: UnsafeCell<()>;
}

#[inline(always)]
pub fn code_end() -> usize {
    unsafe { __code_end.get() as _ }
}

#[inline(always)]
pub fn boot_alloc_start() -> usize {
    unsafe { __boot_alloc_start.get() as _ }
}

#[inline(always)]
pub fn boot_alloc_bitmap_start() -> usize {
    unsafe { __boot_alloc_bitmap_start.get() as _ }
}

#[inline(always)]
pub fn ttbr0_el1_start() -> usize {
    unsafe { __ttbr0_el1_start.get() as _ }
}

#[inline(always)]
pub fn rpi_phys_binary_load_addr() -> usize {
    unsafe { __rpi_phys_binary_load_addr.get() as _ }
}
