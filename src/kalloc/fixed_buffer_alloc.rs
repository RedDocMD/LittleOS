use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::Cell,
    ptr::{self, NonNull},
};

use crate::mmu::align_up;

pub struct FixedSliceAlloc<'a> {
    buf: &'a mut [u8],
    end_idx: Cell<usize>,
}

impl<'a> FixedSliceAlloc<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            end_idx: Cell::new(0),
        }
    }
}

unsafe impl<'a> Allocator for FixedSliceAlloc<'a> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let base = self.buf.as_ptr() as usize;
        let end_idx = self.end_idx.get();
        let ptr_base = align_up(base + end_idx, layout.align());
        if ptr_base + layout.size() - base >= self.buf.len() {
            return Err(AllocError);
        }
        let ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                ptr_base as *mut u8,
                layout.size(),
            ))
        };
        self.end_idx.set(ptr_base + layout.size() - base);
        Ok(ptr)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // nop, cannot deallocate
    }
}
