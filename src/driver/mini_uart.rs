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

const MINI_UART_OFFSET: usize = 0x21_5000;
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
        (0x04 => AUX_ENABLE: ReadWrite<u32, AUX_ENABLE::Register>),
        (0x40 => AUX_MU_IO: ReadWrite<u32>),
        (0x44 => AUX_MU_IER: WriteOnly<u32>),
        (0x48 => AUX_MU_IIR: WriteOnly<u32>),
        (0x4C => AUX_MU_LCR: WriteOnly<u32, AUX_MU_LCR::Register>),
        (0x50 => AUX_MU_MCR: WriteOnly<u32, AUX_MU_MCR::Register>),
        (0x53 => AUX_MU_LSR: ReadOnly<u32, AUX_MU_LSR::Register>),
        (0x60 => AUX_MU_CNTL: WriteOnly<u32, AUX_MU_CNTL::Register>),
        (0x68 => AUX_MU_BAUD: WriteOnly<u32>),
        (0xFF => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

struct MiniUartInner {
    registers: Registers,
}

// use crate::driver::gpio::GPIO_BASE;

impl MiniUartInner {
    // const GPFSEL1: usize = GPIO_BASE + 0x04;
    // const GPPUD: usize = GPIO_BASE + 0x94;
    // const GPPUDCLK0: usize = GPIO_BASE + 0x98;

    // const AUX_ENABLE: usize = MINI_UART_BASE + 0x04;
    // const AUX_MU_CNTL: usize = MINI_UART_BASE + 0x60;
    // const AUX_MU_LCR: usize = MINI_UART_BASE + 0x4C;
    // const AUX_MU_MCR: usize = MINI_UART_BASE + 0x50;
    // const AUX_MU_IER: usize = MINI_UART_BASE + 0x44;
    // const AUX_MU_IIR: usize = MINI_UART_BASE + 0x48;
    // const AUX_MU_BAUD: usize = MINI_UART_BASE + 0x68;
    // const AUX_MU_IO: usize = MINI_UART_BASE + 0x40;
    // const AUX_MU_LSR: usize = MINI_UART_BASE + 0x54;

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
        self.registers.AUX_MU_IIR.set(0xc6);
        self.registers.AUX_MU_BAUD.set(270);

        GPIO.map_uart1_pins();

        self.registers
            .AUX_MU_CNTL
            .write(AUX_MU_CNTL::Tx::SET + AUX_MU_CNTL::Rx::SET);
    }

    // unsafe fn init_unsafe(&mut self) {
    //     use core::ptr::{read, write_volatile};

    //     let val = read(MiniUartInner::AUX_ENABLE as *mut u32);
    //     write_volatile(MiniUartInner::AUX_ENABLE as *mut u32, val | 1);
    //     write_volatile(MiniUartInner::AUX_MU_CNTL as *mut u32, 0);
    //     write_volatile(MiniUartInner::AUX_MU_LCR as *mut u32, 3);
    //     write_volatile(MiniUartInner::AUX_MU_MCR as *mut u32, 0);
    //     write_volatile(MiniUartInner::AUX_MU_IER as *mut u32, 0);
    //     write_volatile(MiniUartInner::AUX_MU_IIR as *mut u32, 0xc6);
    //     write_volatile(MiniUartInner::AUX_MU_BAUD as *mut u32, 270);

    //     let mut val = read(MiniUartInner::GPFSEL1 as *mut u32);
    //     val &= !((7 << 12) | (7 << 15));
    //     val |= (2 << 12) | (2 << 15);
    //     write_volatile(MiniUartInner::GPFSEL1 as *mut u32, val);
    //     write_volatile(MiniUartInner::GPPUD as *mut u32, 0);
    //     crate::cpu::spin_for_cycles(150);
    //     write_volatile(MiniUartInner::GPPUDCLK0 as *mut u32, (1 << 14) | (1 << 15));
    //     crate::cpu::spin_for_cycles(150);
    //     write_volatile(MiniUartInner::GPPUDCLK0 as *mut u32, 0);

    //     write_volatile(MiniUartInner::AUX_MU_CNTL as *mut u32, 3);
    // }

    fn putc(&mut self, c: u8) {
        while self
            .registers
            .AUX_MU_LSR
            .is_set(AUX_MU_LSR::TransmitterEmpty)
        {
            asm::nop();
        }
        self.registers.AUX_MU_IO.set(c as u32);
    }

    // unsafe fn putc_unsafe(&mut self, c: u8) {
    //     use core::ptr::{read, write_volatile};

    //     while read(MiniUartInner::AUX_MU_LSR as *mut u32) & 0x20 == 0 {
    //         asm::nop();
    //     }
    //     write_volatile(MiniUartInner::AUX_MU_IO as *mut u32, c as u32);
    //     crate::qprintln!("Wrote to {:#010X}, {}", MiniUartInner::AUX_MU_IO, c);
    // }
}

impl fmt::Write for MiniUartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.putc(c);
            // unsafe {
            //     self.putc_unsafe(c);
            // }
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
        // self.inner.lock(|inner| unsafe { inner.init_unsafe() });
    }
}

pub static MINI_UART: MiniUart = MiniUart::new();
