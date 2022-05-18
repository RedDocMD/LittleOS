use crate::alloc::{AllocError, Allocator, Layout};
use crate::utils::bitmap::ConstBitmap;

/// BootAllocator is an allocator that is meant to be used only early up
/// in the boot process, before the MMU is setup and we
/// can use proper allocators, like the buddy allocator or
/// slab allocator.
/// BootAllocator will contain exactly **64N** pages, from __data_end.
/// This awkwardness is due to Rust not having stable const generic expressions
/// yet - fingers crosed until then.
pub struct BootAllocator<const N: usize> {
    page_map: ConstBitmap<N>,
    last_page: Option<LastPage>,
}

struct LastPage {
    idx: usize,
    off: Option<usize>,
}

unsafe impl<const N: usize> Allocator for BootAllocator<N> {
    fn allocate(&self, layout: Layout) -> Result<core::ptr::NonNull<[u8]>, AllocError> {
        todo!()
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: Layout) {
        todo!()
    }
}
