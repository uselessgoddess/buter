use std::cell::UnsafeCell;

#[repr(transparent)]
pub(crate) struct SyncUnsafeCell<T: ?Sized> {
    cell: UnsafeCell<T>,
}

impl<T> SyncUnsafeCell<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            cell: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> SyncUnsafeCell<T> {
    #[inline]
    pub fn get(&self) -> *mut T {
        self.cell.get()
    }
}

unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}
