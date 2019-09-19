use crate::{failed, ComInterface};

use std::ptr::NonNull;

use super::*;
use std::marker::PhantomData;
use winapi::ctypes::c_void;
use winapi::shared::winerror::{E_NOINTERFACE, E_POINTER};

pub struct ComPtr<T: ?Sized + ComInterface> {
    ptr: NonNull<c_void>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized + ComInterface> ComPtr<T> {
    /// Creates a new ComPtr that comforms to the interface T
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid VTable for the Interface T
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null
    pub unsafe fn new(ptr: *mut c_void) -> ComPtr<T> {
        ComPtr {
            ptr: NonNull::new(ptr).expect("ptr was null"),
            phantom: PhantomData,
        }
    }

    pub fn into_raw(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    pub fn get_ptr(&self) -> NonNull<c_void> {
        self.ptr
    }

    pub fn get_interface<S: ComInterface + ?Sized>(&self) -> Option<ComPtr<S>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let hr = unsafe { self.query_interface(&S::IID as *const IID, &mut ppv) };
        if failed(hr) {
            assert!(
                hr == E_NOINTERFACE || hr == E_POINTER,
                "QueryInterface returned non-standard error"
            );
            return None;
        }
        assert!(!ppv.is_null(), "The pointer to the interface returned from a successful call to QueryInterface was null");
        unsafe { Some(ComPtr::new(ppv)) }
    }
}

impl<T: ComInterface + ?Sized> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            self.release();
        }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        let new_ptr = ComPtr {
            ptr: self.ptr,
            phantom: PhantomData,
        };
        new_ptr.add_ref();
        new_ptr
    }
}
