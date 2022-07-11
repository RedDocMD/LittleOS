use core::{alloc::Allocator, cell::UnsafeCell};

use crate::{
    driver::mailbox::{Mailbox, PropertyTag},
    error::OsError,
};

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

#[repr(C)]
struct Mem {
    base: u32,
    size: u32,
}

struct GetArmMemory;

struct GetVcMemory;

unsafe impl PropertyTag for GetArmMemory {
    type RecvType = Mem;

    fn identifier(&self) -> u32 {
        0x0001_0005
    }
}

unsafe impl PropertyTag for GetVcMemory {
    type RecvType = Mem;

    fn identifier(&self) -> u32 {
        0x0001_0006
    }
}

pub struct MemLimits {
    pub arm_base: usize,
    pub arm_size: usize,
    pub vc_base: usize,
    pub vc_size: usize,
}

pub fn get_memory_limits<A: Allocator>(alloc: &A) -> Result<MemLimits, OsError> {
    let mut mbox = Mailbox::new(alloc)?;
    mbox.append_tag(GetVcMemory)?;
    mbox.append_tag(GetArmMemory)?;
    mbox.call()?;
    let vc_mem = mbox.read_tag_result::<GetVcMemory>(0).unwrap();
    let arm_mem = mbox.read_tag_result::<GetArmMemory>(1).unwrap();

    Ok(MemLimits {
        arm_base: arm_mem.base as usize,
        arm_size: arm_mem.size as usize,
        vc_base: vc_mem.base as usize,
        vc_size: vc_mem.size as usize,
    })
}
