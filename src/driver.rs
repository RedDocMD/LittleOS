mod qemu;

pub fn console() -> impl core::fmt::Write {
    qemu::QEMUOutput {}
}
