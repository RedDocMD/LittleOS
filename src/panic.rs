use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::kprintln!("******* Kernel Panic *********");
    crate::kprintln!("{}", info);
    crate::kprintln!("******* xxxxxxxxxxxx *********");
    loop {}
}
