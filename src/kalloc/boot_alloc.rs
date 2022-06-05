use core::{
    cell::RefCell,
    ptr::{self, NonNull},
};

use crate::{
    kalloc::{AllocError, Allocator, Layout},
    mmu::{
        layout::{boot_alloc_start, data_end},
        PAGE_SIZE,
    },
};

use bitvec::slice::BitSlice;

const BOOT_ALLOC_SPACE: usize = 16 * (1 << 20);
const BOOT_ALLOC_PAGE_COUNT: usize = BOOT_ALLOC_SPACE / PAGE_SIZE;
const BITMAP_LEN: usize = BOOT_ALLOC_PAGE_COUNT / 8;

/// BootAllocator is an allocator that is meant to be used only early up
/// in the boot process, before the MMU is setup and we
/// can use proper allocators, like the buddy allocator or
/// slab allocator.
pub struct BootAllocator {
    page_map: RefCell<&'static mut BitSlice<u64>>,
    last_page: Option<LastPage>,
}

struct LastPage {
    idx: usize,
    off: Option<usize>,
}

impl BootAllocator {
    pub fn new() -> Self {
        let slice = ptr::slice_from_raw_parts_mut(data_end() as *mut u64, BITMAP_LEN);
        let mut long_words = NonNull::new(slice).unwrap();
        let long_words_slice = unsafe { long_words.as_mut() };
        let bitmap = BitSlice::from_slice_mut(long_words_slice);
        bitmap.fill(false);
        Self {
            page_map: RefCell::new(bitmap),
            last_page: None,
        }
    }
}

unsafe impl Allocator for BootAllocator {
    fn allocate(&self, layout: Layout) -> Result<core::ptr::NonNull<[u8]>, AllocError> {
        let size = layout.size();
        let alignment = layout.align();
        if size > PAGE_SIZE || alignment > PAGE_SIZE {
            todo!("Implement proper allocator for general size and alignment");
        }
        let idx = self.page_map.borrow().first_zero();
        if let Some(idx) = idx {
            self.page_map.borrow_mut().set(idx, true);
            let addr = boot_alloc_start() + idx * PAGE_SIZE;
            let slice = ptr::slice_from_raw_parts_mut(addr as *mut u8, size);
            return Ok(NonNull::new(slice).unwrap());
        }
        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
        let size = layout.size();
        let alignment = layout.align();
        if size > PAGE_SIZE || alignment > PAGE_SIZE {
            todo!("Implement proper allocator for general size and alignment");
        }
        let addr = ptr.as_ptr() as usize;
        let idx = (addr - boot_alloc_start()) / PAGE_SIZE;
        self.page_map.borrow_mut().set(idx, false);
    }
}
