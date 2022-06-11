use cortex_a::asm;
use tock_registers::interfaces::Readable;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}

#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

pub fn current_el() -> Option<u8> {
    use cortex_a::registers;

    registers::CurrentEL
        .read_as_enum::<registers::CurrentEL::EL::Value>(registers::CurrentEL::EL)
        .map(|el| match el {
            registers::CurrentEL::EL::Value::EL0 => 0,
            registers::CurrentEL::EL::Value::EL1 => 1,
            registers::CurrentEL::EL::Value::EL2 => 2,
            registers::CurrentEL::EL::Value::EL3 => 3,
        })
}
