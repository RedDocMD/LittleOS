use core::arch::global_asm;

global_asm!(
    include_str!("boot.S"),
    CONST_CORE_ID_MASK = const 0b11
);

#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init();
}
