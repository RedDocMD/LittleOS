use crate::{cpu, sync::NullLock};

use super::mmio::{MMIODerefWrapper, MMIO_BASE};
use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadWrite, WriteOnly},
};

const GPIO_OFFSET: usize = 0x0020_0000;
pub const GPIO_BASE: usize = MMIO_BASE + GPIO_OFFSET;

register_bitfields! {
    u32,

    GPFSEL1 [
        FSEL14 OFFSET(12) NUMBITS(3) [
            Alt0 = 0b100,
            Alt5 = 0b010,
        ],
        FSEL15 OFFSET(15) NUMBITS(3) [
            Alt0 = 0b100,
            Alt5 = 0b010,
        ],
    ],

    GPPUD [
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10,
        ]
    ],

    GPPUDCLK0 [
        PUDCLK14 14,
        PUDCLK15 15,
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: WriteOnly<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: WriteOnly<u32, GPPUDCLK0::Register>),
        (0x9C => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

struct GpioInner {
    registers: Registers,
}

impl GpioInner {
    const fn new() -> Self {
        Self {
            registers: unsafe { Registers::new(GPIO_BASE) },
        }
    }

    fn disable_pull_up_down(&mut self) {
        self.registers.GPPUD.write(GPPUD::PUD::Off);
        cpu::spin_for_cycles(150);
        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK14::SET + GPPUDCLK0::PUDCLK15::SET);
        cpu::spin_for_cycles(150);
        self.registers.GPPUDCLK0.set(0);
    }

    fn map_uart1_pins(&mut self) {
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL14::Alt5 + GPFSEL1::FSEL15::Alt5);
        self.disable_pull_up_down();
    }
}

pub struct Gpio {
    inner: NullLock<GpioInner>,
}

impl Gpio {
    const fn new() -> Self {
        Self {
            inner: NullLock::new(GpioInner::new()),
        }
    }

    pub fn map_uart1_pins(&self) {
        self.inner.lock(|inner| inner.map_uart1_pins());
    }
}

pub static GPIO: Gpio = Gpio::new();
