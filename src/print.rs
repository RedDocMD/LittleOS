use core::fmt;

use crate::driver::{console, qemu_console};

pub trait Write {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    console().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _qemu_print(args: fmt::Arguments) {
    qemu_console().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! qprint {
    ($($arg:tt)*) => ($crate::print::_qemu_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! qprintln {
    () => ($crate::qprint!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_qemu_print(format_args_nl!($($arg)*));
    })
}
