use core::fmt::{self, Write};

use crate::{print, sync::NullLock};

use super::{
    gpio::GPIO,
    mmio::{MMIODerefWrapper, MMIO_BASE},
    DeviceDriver,
};
use cortex_a::asm;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

const MINI_UART_OFFSET: usize = 0x0021_5000;
const MINI_UART_BASE: usize = MMIO_BASE + MINI_UART_OFFSET;

register_bitfields! {
    u32,

    AUX_ENABLE [
        Spi2 2,
        Spi1 1,
        MiniUart 0,
    ],

    AUX_MU_LCR [
        DataSize OFFSET(0) NUMBITS(2) [
            Mode7Bit = 0b00,
            Mode8Bit = 0b11,
        ]
    ],

    AUX_MU_MCR [
        RTS 1,
    ],

    AUX_MU_CNTL [
        CTS OFFSET(7) NUMBITS(1),
        RTS OFFSET(6) NUMBITS(1),
        RTS_AUTO OFFSET(4) NUMBITS(2),
        Transmit_CTS_AUTO OFFSET(3) NUMBITS(1),
        Transmit_RTS_AUTO OFFSET(2) NUMBITS(1),
        Tx OFFSET(1) NUMBITS(1),
        Rx OFFSET(0) NUMBITS(1),
    ],

    AUX_MU_LSR [
        TransmitterIdle 6,
        TransmitterEmpty 5,
        ReceiverOverrun 1,
        DataReady 0,
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => AUX_ENABLE: ReadWrite<u32, AUX_ENABLE::Register>),
        (0x08 => _reserved2),
        (0x40 => AUX_MU_IO: ReadWrite<u32>),
        (0x44 => AUX_MU_IER: WriteOnly<u32>),
        (0x48 => AUX_MU_IIR: WriteOnly<u32>),
        (0x4C => AUX_MU_LCR: WriteOnly<u32, AUX_MU_LCR::Register>),
        (0x50 => AUX_MU_MCR: WriteOnly<u32, AUX_MU_MCR::Register>),
        (0x54 => AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>),
        (0x58 => _reserved3),
        (0x60 => AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>),
        (0x64 => _reserved4),
        (0x68 => AUX_MU_BAUD: WriteOnly<u32>),
        (0x6C => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

struct MiniUartInner {
    registers: Registers,
}

impl MiniUartInner {
    const fn new() -> Self {
        Self {
            registers: unsafe { Registers::new(MINI_UART_BASE) },
        }
    }

    fn init(&mut self) {
        // Init UART
        self.registers.AUX_ENABLE.modify(AUX_ENABLE::MiniUart::SET);
        self.registers
            .AUX_MU_CNTL
            .write(AUX_MU_CNTL::Tx::CLEAR + AUX_MU_CNTL::Rx::CLEAR);
        self.registers
            .AUX_MU_LCR
            .write(AUX_MU_LCR::DataSize::Mode8Bit);
        self.registers.AUX_MU_MCR.write(AUX_MU_MCR::RTS::CLEAR);
        self.registers.AUX_MU_IER.set(0);
        self.registers.AUX_MU_BAUD.set(270);

        GPIO.map_uart1_pins();

        self.registers
            .AUX_MU_CNTL
            .write(AUX_MU_CNTL::Tx::SET + AUX_MU_CNTL::Rx::SET);
    }

    fn putc(&mut self, c: u8) {
        while !self
            .registers
            .AUX_MU_LSR
            .matches_all(AUX_MU_LSR::TransmitterEmpty::SET)
        {
            asm::nop();
        }
        self.registers.AUX_MU_IO.set(c as u32);
    }
}

impl fmt::Write for MiniUartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.putc(c);
        }
        Ok(())
    }
}

pub struct MiniUart {
    inner: NullLock<MiniUartInner>,
}

impl MiniUart {
    const fn new() -> Self {
        Self {
            inner: NullLock::new(MiniUartInner::new()),
        }
    }
}

impl print::Write for MiniUart {
    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| inner.write_fmt(args))
    }
}

impl DeviceDriver for MiniUart {
    fn init(&self) {
        self.inner.lock(|inner| inner.init());
    }
}

pub static MINI_UART: MiniUart = MiniUart::new();
