// An issue with having T be Human is that I am never
// actually possessing the entire Human struct, just
// an interface pointer.
use crate::iunknown::RawIUnknown;

use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::NonNull;

pub struct ComPtr<T> {
    ptr: NonNull<T>,
}

impl<T> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> ComPtr<T> {
    /// NonNull<T> must be safely convertable to *mut RawIUnknown
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        ComPtr { ptr }
    }

    fn add_ref(&self) {
        unsafe { (*(self.ptr.as_ptr() as *mut RawIUnknown)).raw_add_ref() };
    }
}

impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        self.add_ref();
        ComPtr { ptr: self.ptr }
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            (*(self.ptr.as_ptr() as *mut RawIUnknown)).raw_release();
        }
    }
}
