use core::{
    cell::RefCell,
    ptr::{self, NonNull},
};

use crate::{
    alloc::{AllocError, Allocator, Layout},
    mmu::{layout::boot_alloc_start, PAGE_SIZE},
};

mod bitmap {
    use core::ptr::{self, NonNull};

    use crate::mmu::{layout::data_end, PAGE_SIZE};

    const BOOT_ALLOC_SPACE: usize = 16 * (1 << 20);
    const BOOT_ALLOC_PAGE_COUNT: usize = BOOT_ALLOC_SPACE / PAGE_SIZE;
    pub const BITMAP_LEN: usize = BOOT_ALLOC_PAGE_COUNT / 8;

    pub struct Bitmap {
        long_words: NonNull<[u64]>,
    }

    impl Bitmap {
        pub fn new() -> Self {
            let slice = ptr::slice_from_raw_parts_mut(data_end() as *mut u64, BITMAP_LEN);
            let mut long_words = NonNull::new(slice).unwrap();
            let long_words_slice = unsafe { long_words.as_mut() };
            long_words_slice.fill(0);
            Self { long_words }
        }

        fn get_arr(&self, idx: usize) -> u64 {
            let slice = unsafe { self.long_words.as_ref() };
            slice[idx]
        }

        fn set_arr(&mut self, idx: usize, val: u64) {
            let slice = unsafe { self.long_words.as_mut() };
            slice[idx] = val;
        }

        pub fn test(&self, bit_idx: usize) -> bool {
            let word = self.get_arr(long_word_idx(bit_idx));
            (word >> long_word_offset(bit_idx)) & 1 != 0
        }

        pub fn set(&mut self, bit_idx: usize) {
            let idx = long_word_idx(bit_idx);
            self.set_arr(idx, self.get_arr(idx) | (1 << long_word_offset(bit_idx)));
        }

        pub fn clear(&mut self, bit_idx: usize) {
            let idx = long_word_idx(bit_idx);
            self.set_arr(idx, self.get_arr(idx) & !(1 << long_word_offset(bit_idx)));
        }

        pub fn first_clear_idx(&self) -> Option<usize> {
            for i in 0..BITMAP_LEN {
                let word = self.get_arr(i);
                if word == 0xFFFF_FFFF_FFFF_FFFF {
                    continue;
                }
                for off in 0..64 {
                    if (word >> off) & 1 == 0 {
                        return Some(i * 64 + off);
                    }
                }
                unreachable!("We should have had a clear bit");
            }
            None
        }
    }

    #[inline(always)]
    fn long_word_idx(bit_idx: usize) -> usize {
        bit_idx / 64
    }

    #[inline(always)]
    fn long_word_offset(bit_idx: usize) -> usize {
        bit_idx % 64
    }
}

/// BootAllocator is an allocator that is meant to be used only early up
/// in the boot process, before the MMU is setup and we
/// can use proper allocators, like the buddy allocator or
/// slab allocator.
pub struct BootAllocator {
    page_map: RefCell<bitmap::Bitmap>,
    last_page: Option<LastPage>,
}

struct LastPage {
    idx: usize,
    off: Option<usize>,
}

impl BootAllocator {
    pub fn new() -> Self {
        Self {
            page_map: RefCell::new(bitmap::Bitmap::new()),
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
        let idx = self.page_map.borrow().first_clear_idx();
        if let Some(idx) = idx {
            self.page_map.borrow_mut().set(idx);
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
        self.page_map.borrow_mut().clear(idx);
    }
}
