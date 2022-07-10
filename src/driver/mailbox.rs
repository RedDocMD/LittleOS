use core::{
    alloc::{Allocator, Layout},
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

use cortex_a::asm;
use std_alloc::vec::Vec;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, WriteOnly},
};

use crate::{driver::mmio::MMIO_BASE, error::OsError, mmu::align_up};

use super::mmio::MMIODerefWrapper;

const VIDEOCORE_MBOX_OFFSET: usize = 0x0000_B880;
const VIDEOCORE_MBOX_BASE: usize = MMIO_BASE + VIDEOCORE_MBOX_OFFSET;

register_bitfields! {
    u32,

    Mbox0_Status [
        FULL 31,
        EMPTY 30,
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => Mbox0_Read: ReadOnly<u32>),
        (0x04 => _reserved1),
        (0x18 => Mbox0_Status: ReadOnly<u32, Mbox0_Status::Register>),
        (0x1C => _reserved2),
        (0x20 => Mbox1_Write: WriteOnly<u32>),
        (0x24 => @END),
    }
}

pub struct Mailbox<'a, A: Allocator> {
    allocator: &'a A,
    buffer: NonNull<u8>,
    cap: usize,
    len: usize,
    tag_idx: Vec<usize, &'a A>,
    has_result: bool,
    registers: MMIODerefWrapper<RegisterBlock>,
}

const DEFAULT_MAILBOX_SIZE: usize = 36 * mem::size_of::<u32>();

impl<'a, A: Allocator> Mailbox<'a, A> {
    pub fn new(allocator: &'a A) -> Result<Self, OsError> {
        let layout = Layout::array::<u8>(DEFAULT_MAILBOX_SIZE)?.align_to(16)?;
        let buffer = allocator.allocate(layout)?;
        let tag_idx = Vec::new_in(allocator);
        let registers = unsafe { MMIODerefWrapper::new(VIDEOCORE_MBOX_BASE) };
        let mut mailbox = Self {
            allocator,
            buffer: buffer.cast(),
            cap: DEFAULT_MAILBOX_SIZE,
            len: 8,
            tag_idx,
            registers,
            has_result: false,
        };
        mailbox.write_value(0, 0u32);
        mailbox.write_value(4, 0u32);
        Ok(mailbox)
    }

    fn resize(&mut self, new_cap: usize) -> Result<(), OsError> {
        let layout = Layout::array::<u8>(new_cap)?.align_to(16)?;
        let new_buffer = self.allocator.allocate(layout)?;
        unsafe { ptr::copy(self.buffer.as_ptr(), new_buffer.cast().as_ptr(), self.len) };

        let old_layout = Layout::array::<u8>(self.cap)?.align_to(16)?;
        unsafe { self.allocator.deallocate(self.buffer, old_layout) };

        self.buffer = new_buffer.cast();
        self.cap = new_cap;
        Ok(())
    }

    fn append_value<T>(&mut self, value: T) -> Result<(), OsError> {
        let val_size = mem::size_of::<T>();
        while self.len + val_size > self.cap {
            self.resize(self.cap * 2)?;
        }
        unsafe {
            let end = self.buffer.as_ptr().add(self.len);
            ptr::copy(ptr::addr_of!(value) as *const _, end, val_size);
        }
        self.len += val_size;
        Ok(())
    }

    fn write_value<T>(&mut self, idx: usize, value: T) {
        let val_size = mem::size_of::<T>();
        assert!(idx + val_size <= self.cap);
        unsafe {
            let buf_ptr = self.buffer.as_ptr().add(idx);
            ptr::copy(ptr::addr_of!(value) as *const _, buf_ptr, val_size);
        }
    }

    fn read_value<T>(&self, idx: usize) -> T {
        let val_size = mem::size_of::<T>();
        assert!(idx + val_size <= self.cap);
        let mut val: MaybeUninit<T> = MaybeUninit::uninit();
        unsafe {
            let buf_ptr = self.buffer.as_ptr().add(idx);
            ptr::copy(buf_ptr, val.as_mut_ptr() as *mut _, val_size);
            val.assume_init()
        }
    }

    fn pop_value<T>(&mut self) {
        let val_size = mem::size_of::<T>();
        assert!(val_size <= self.len);
        self.len -= val_size;
    }

    pub fn append_tag<T: PropertyTag>(&mut self, tag: T) -> Result<(), OsError> {
        self.has_result = false;
        self.tag_idx.push(self.len);

        self.append_value(tag.identifier())?;
        let buf = tag.send_buffer();
        let buf_len = buf.len();
        let recv_buf_len = mem::size_of::<T::RecvType>();
        let full_len = buf_len.max(recv_buf_len);
        self.append_value(full_len as u32)?;
        self.append_value(0u32)?;
        for i in buf {
            self.append_value(*i)?;
        }
        if recv_buf_len > buf_len {
            for _ in 0..(recv_buf_len - buf_len) {
                self.append_value(0u8)?;
            }
        }
        let pad_len = align_up(full_len, 4) - full_len;
        for _ in 0..pad_len {
            self.append_value(0u8)?;
        }

        Ok(())
    }

    pub fn read_tag_result<T: PropertyTag>(&self, tag_idx: usize) -> Option<T::RecvType> {
        if !self.has_result {
            return None;
        }
        let tag_buf_idx = *self.tag_idx.get(tag_idx)?;
        let resp_code: u32 = self.read_value(tag_buf_idx + 8);
        if (resp_code >> 31) != 1 {
            return None;
        }
        let recv_type_size = mem::size_of::<T::RecvType>();
        assert!((resp_code & 0x7FFF_FFFF) == recv_type_size as u32);
        Some(self.read_value(tag_buf_idx + 12))
    }

    pub fn call(&mut self) -> Result<bool, OsError> {
        // Push end tag
        self.append_value(0u32)?;
        // Set length
        self.write_value(0, self.len as u32);
        let value = (self.addr() | 0x8) as u32;

        const MBOX_RESPONSE: u32 = 0x8000_0000;

        // Wait until we can write mailbox
        while self
            .registers
            .Mbox0_Status
            .matches_all(Mbox0_Status::FULL::SET)
        {
            asm::nop();
        }

        // Write mailbox
        self.registers.Mbox1_Write.set(value);

        // Now wait for response
        loop {
            while self
                .registers
                .Mbox0_Status
                .matches_all(Mbox0_Status::EMPTY::SET)
            {
                asm::nop();
            }
            if self.registers.Mbox0_Read.get() == value {
                let resp: u32 = self.read_value(4);
                self.has_result = resp == MBOX_RESPONSE;
                break;
            }
        }

        // Pop end tag
        self.pop_value::<u32>();
        Ok(self.has_result)
    }

    fn addr(&self) -> usize {
        (self.buffer.as_ptr() as usize) & !0xF
    }
}

impl<'a, A: Allocator> Drop for Mailbox<'a, A> {
    fn drop(&mut self) {
        let layout = Layout::array::<u8>(self.cap).unwrap().align_to(16).unwrap();
        unsafe { self.allocator.deallocate(self.buffer, layout) };
    }
}

/// # Safety
///
/// This is a wildly unsafe trait to use - no compiler guarantees!
/// RecvType must be the proper receiver type for this PropertyTag.
/// Also ensure it is repr(C).
pub unsafe trait PropertyTag {
    type RecvType;

    fn identifier(&self) -> u32;

    fn send_buffer(&self) -> &[u8] {
        unsafe { &*ptr::slice_from_raw_parts(self as *const Self as _, mem::size_of_val(self)) }
    }
}
