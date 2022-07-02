use core::alloc::{AllocError, LayoutError};

#[derive(Debug)]
pub enum OsError {
    Alloc(AllocError),
    Layout(LayoutError),
}

impl From<AllocError> for OsError {
    fn from(err: AllocError) -> Self {
        OsError::Alloc(err)
    }
}

impl From<LayoutError> for OsError {
    fn from(err: LayoutError) -> Self {
        OsError::Layout(err)
    }
}
