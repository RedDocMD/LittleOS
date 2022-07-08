pub mod layout;
pub mod paging;

pub const PAGE_SIZE_ORDER: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_ORDER;

pub const TOTAL_MEMORY: usize = 512 << 20;

pub const fn align_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

pub const fn align_up(value: usize, align: usize) -> usize {
    align_down(value + align - 1, align)
}

pub const fn is_aligned(value: usize, align: usize) -> bool {
    value & (align - 1) == 0
}
