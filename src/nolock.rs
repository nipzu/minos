use core::cell::UnsafeCell;

#[repr(C)]
pub struct NoLock<T> {
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for NoLock<T> {}

impl<T> NoLock<T> {
    pub const fn new(value: T) -> NoLock<T> {
        NoLock {
            value: UnsafeCell::new(value),
        }
    }

    /// Safety:
    /// - the value must be initialized before calling this
    /// - only one mutable reference at any time
    pub unsafe fn lock(&self) -> &mut T {
        &mut *self.value.get()
    }
}
