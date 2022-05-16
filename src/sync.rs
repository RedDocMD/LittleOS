use core::cell::UnsafeCell;

pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

/// Safety: This is safe only when the kernel is uniprocessor.
/// Replace with spinlock when MMU is enabled.
unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}

/// Safety: This is safe only when the kernel is uniprocessor.
/// Replace with spinlock when MMU is enabled.
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    /// Safety: This is safe only when the kernel is uniprocessor.
    /// Replace with spinlock when MMU is enabled.
    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };
        f(data)
    }
}
