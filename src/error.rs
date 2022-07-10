use core::{
    alloc::{AllocError, LayoutError},
    fmt,
};

#[derive(Debug)]
pub enum OsError {
    Alloc(AllocError),
    Layout(LayoutError),
    InvalidDepth(u32),
    FramebufferNotAllocated,
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

impl fmt::Display for OsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OsError::Alloc(err) => write!(f, "{}", err),
            OsError::Layout(err) => write!(f, "{}", err),
            OsError::InvalidDepth(depth) => write!(f, "depth of {} not supported", depth),
            OsError::FramebufferNotAllocated => write!(f, "failed to allocate framebuffer"),
        }
    }
}
