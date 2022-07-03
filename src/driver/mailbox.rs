use core::{
    alloc::{Allocator, Layout},
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};

use std_alloc::vec::Vec;

use crate::{error::OsError, mmu::align_up};

pub struct Mailbox<'a, A: Allocator> {
    allocator: &'a A,
    buffer: NonNull<u8>,
    cap: usize,
    len: usize,
    tag_idx: Vec<usize, &'a A>,
    has_result: bool,
}

const DEFAULT_MAILBOX_SIZE: usize = 144;

impl<'a, A: Allocator> Mailbox<'a, A> {
    pub fn new(allocator: &'a A) -> Result<Self, OsError> {
        let layout = Layout::array::<u8>(DEFAULT_MAILBOX_SIZE)?.align_to(16)?;
        let buffer = allocator.allocate(layout)?;
        let tag_idx = Vec::new_in(allocator);
        let mut mailbox = Self {
            allocator,
            buffer: buffer.cast(),
            cap: DEFAULT_MAILBOX_SIZE,
            len: 2,
            tag_idx,
            has_result: false,
        };
        mailbox.write_value(0u32, 0);
        mailbox.write_value(0u32, 4);
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

    fn write_value<T>(&mut self, value: T, idx: usize) {
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

    pub fn append_tag<T: PropertyTag>(&mut self, tag: T) -> Result<(), OsError> {
        self.has_result = false;
        self.append_value(tag.identifier())?;
        let buf = tag.send_buffer();
        let buf_len = buf.len();
        self.append_value(buf_len as u32)?;
        self.append_value(0u32)?;
        for i in buf {
            self.append_value(i)?;
        }
        let recv_buf_len = mem::size_of::<T::RecvType>();
        if recv_buf_len > buf_len {
            for _ in 0..(recv_buf_len - buf_len) {
                self.append_value(0)?;
            }
        }
        let pad_len = align_up(buf_len.max(recv_buf_len), 4) - buf_len;
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
        assert!(self.read_value::<u32>(tag_buf_idx + 4) == recv_type_size as u32);
        Some(self.read_value(tag_buf_idx + 12))
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

pub mod tags {
    use super::PropertyTag;

    pub struct GetVcMem;

    #[repr(C)]
    #[derive(Debug)]
    pub struct VcMem {
        base: u32,
        size: u32,
    }

    unsafe impl PropertyTag for GetVcMem {
        type RecvType = VcMem;

        fn identifier(&self) -> u32 {
            0x0001_0006
        }
    }
}
