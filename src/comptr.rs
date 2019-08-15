// An issue with having T be Human is that I am never
// actually possessing the entire Human struct, just
// an interface pointer.
use crate::ComInterface;

use std::ptr::NonNull;

use super::*;
use std::marker::PhantomData;
use winapi::ctypes::c_void;

pub struct ComPtr<T: ComInterface + ?Sized> {
    ptr: NonNull<c_void>,
    phantom: PhantomData<T>,
}

impl<T: ComInterface + ?Sized> ComPtr<T> {
    /// NonNull<T> must be safely convertable to *mut RawIUnknown
    pub fn new(ptr: NonNull<c_void>) -> Self {
        ComPtr {
            ptr,
            phantom: PhantomData,
        }
    }

    pub fn into_raw(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    pub fn get_ptr(&self) -> NonNull<c_void> {
        self.ptr
    }

    fn cast_and_add_ref(&self) {
        unsafe { (*(self as *const _ as *const ComPtr<IUnknown>)).add_ref(); }
    }
}

impl<T: ComInterface + ?Sized> Drop for ComPtr<T> {
    fn drop(&mut self) {
        println!("Dropped!");
        unsafe {
            (*(self as *const _ as *const ComPtr<IUnknown>)).release();
        }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        self.cast_and_add_ref();
        ComPtr {
            ptr: self.ptr,
            phantom: PhantomData
        }
    }
}