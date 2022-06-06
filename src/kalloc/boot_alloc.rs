use core::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    ptr::{self, NonNull},
};

use crate::{
    kalloc::{AllocError, Allocator, Layout},
    mmu::{
        align_up, is_aligned,
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
    last_page: Cell<Option<LastPage>>,
}

#[derive(Clone, Copy)]
struct LastPage {
    idx: usize,
    off: usize,
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
            last_page: Cell::new(None),
        }
    }

    fn last_page(&self) -> Option<LastPage> {
        self.last_page.get()
    }
}

unsafe impl Allocator for BootAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let size = layout.size();
        let alignment = layout.align();
        if alignment > PAGE_SIZE {
            unimplemented!("Cannot handle alignment greater than {}", PAGE_SIZE);
        }
        let actual_size = align_up(size, alignment);
        let page_cnt = align_up(actual_size, PAGE_SIZE) / PAGE_SIZE;
        let residual_size = page_cnt * PAGE_SIZE - actual_size;

        let page_idx = find_page_idx(&self.page_map.borrow(), page_cnt);
        if let Some(page_idx) = page_idx {
            let mut bitmap = self.page_map.borrow_mut();
            if page_cnt > 1 {
                bitmap[page_idx..page_cnt - 1].fill(true);
            }

            // Special case - try to squeeze into last page
            let shift = shift_from_last_page(&self.last_page(), page_idx, alignment);
            if let Some(shift) = shift {
                match shift.cmp(&residual_size) {
                    Ordering::Less => {
                        bitmap.set(page_idx + page_cnt - 1, true);
                        let new_residual_size = residual_size - shift;
                        self.last_page.set(Some(LastPage {
                            idx: page_idx + page_cnt - 1,
                            off: new_residual_size,
                        }));
                    }
                    Ordering::Equal => {
                        self.last_page.set(None);
                    }
                    Ordering::Greater => {
                        let new_residual_size = PAGE_SIZE - (shift - residual_size);
                        self.last_page.set(Some(LastPage {
                            idx: page_idx + page_cnt - 2,
                            off: new_residual_size,
                        }));
                    }
                }

                let addr = boot_alloc_start() + page_idx * PAGE_SIZE - shift;
                let ptr = ptr::slice_from_raw_parts_mut(addr as *mut u8, size);
                return Ok(NonNull::new(ptr).unwrap());
            }

            // Otherwise fallback case
            bitmap.set(page_idx + page_cnt - 1, true);
            if residual_size != 0 {
                self.last_page.set(Some(LastPage {
                    idx: page_idx + page_cnt - 1,
                    off: actual_size,
                }));
            } else {
                self.last_page.set(None);
            }

            let addr = boot_alloc_start() + page_idx * PAGE_SIZE;
            let ptr = ptr::slice_from_raw_parts_mut(addr as *mut u8, size);
            return Ok(NonNull::new(ptr).unwrap());
        }
        Err(AllocError)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size();
        let alignment = layout.align();
        if alignment > PAGE_SIZE {
            unreachable!("Cannot handle alignment greater than {}", PAGE_SIZE);
        }
        let actual_size = align_up(size, alignment);
        let start_addr = ptr.as_ptr() as usize;
        if !is_aligned(start_addr, PAGE_SIZE) {
            // This means we were squeezed partially into the last page.
            // Leave that as allocated.
            let next_page_addr = align_up(start_addr, PAGE_SIZE);
            let shift = next_page_addr - start_addr;
            let new_actual_size = actual_size - shift;
            let new_page_cnt = new_actual_size / PAGE_SIZE;
            let mut bitmap = self.page_map.borrow_mut();
            if new_page_cnt > 0 {
                let new_page_idx = (next_page_addr - boot_alloc_start()) / PAGE_SIZE;
                bitmap[new_page_idx..(new_page_idx + new_page_cnt)].fill(false);
                // TODO: Put better logic for last_page update
            } else if let Some(last_page) = self.last_page() {
                let page_idx = (start_addr - boot_alloc_start()) / PAGE_SIZE;
                if last_page.idx == page_idx && last_page.off == start_addr + actual_size {
                    self.last_page.set(Some(LastPage {
                        idx: last_page.idx,
                        off: start_addr,
                    }));
                }
            }
        } else {
            // If the last page was partially allocated, leave it marked as allocated.
            let page_cnt = actual_size / PAGE_SIZE;
            let page_idx = (start_addr - boot_alloc_start()) / PAGE_SIZE;
            let mut bitmap = self.page_map.borrow_mut();
            if page_cnt > 0 {
                bitmap[page_idx..(page_idx + page_cnt)].fill(false);
                let residual_size = actual_size - page_cnt * PAGE_SIZE;
                if residual_size > 0 {
                    let last_page_idx = page_idx + page_cnt;
                    if let Some(last_page) = self.last_page() {
                        if last_page.idx == last_page_idx && last_page.off == residual_size {
                            bitmap.set(last_page_idx, false);
                            self.last_page.set(None);
                        }
                    }
                }
            } else if let Some(last_page) = self.last_page() {
                if last_page.idx == page_idx && last_page.off == actual_size {
                    bitmap.set(page_idx + page_cnt, false);
                    self.last_page.set(None);
                }
            }
        }
    }
}

fn find_page_idx(bitmap: &BitSlice<u64>, page_cnt: usize) -> Option<usize> {
    for (idx, window) in bitmap.windows(page_cnt).enumerate() {
        if window.not_any() {
            return Some(idx);
        }
    }
    None
}

fn shift_from_last_page(
    last_page: &Option<LastPage>,
    page_idx: usize,
    alignment: usize,
) -> Option<usize> {
    if let Some(last_page) = last_page {
        if last_page.idx == page_idx - 1 {
            let last_ins_addr = align_up(
                boot_alloc_start() + last_page.idx * PAGE_SIZE + last_page.off,
                alignment,
            );
            let curr_ins_addr = boot_alloc_start() + page_idx * PAGE_SIZE;
            let shift = curr_ins_addr - last_ins_addr;
            if shift == 0 {
                return None;
            } else {
                return Some(shift);
            }
        }
    }
    None
}
