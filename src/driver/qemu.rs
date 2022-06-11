#![allow(dead_code)]

use core::{
    fmt::{self, Write},
    ptr,
};

use crate::{print, sync::NullLock};

pub struct QEMUOutputInner;
pub struct QEMUOutput {
    inner: NullLock<QEMUOutputInner>,
}

impl QEMUOutput {
    const fn new() -> Self {
        Self {
            inner: NullLock::new(QEMUOutputInner {}),
        }
    }
}

pub static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe {
                ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
            }
        }
        Ok(())
    }
}

impl print::Write for QEMUOutput {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| inner.write_fmt(args))
    }
}
