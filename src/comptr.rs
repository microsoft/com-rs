use crate::{failed, ComInterface};

use std::ptr::NonNull;

use super::*;
use std::marker::PhantomData;
use winapi::ctypes::c_void;
use winapi::shared::winerror::E_NOINTERFACE;

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

    fn cast_and_add_ref(&mut self) {
        unsafe {
            (*(self as *const _ as *mut ComPtr<dyn IUnknown>)).add_ref();
        }
    }

    pub fn get_interface<S: ComInterface + ?Sized>(&mut self) -> Option<ComPtr<S>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let hr = unsafe {
            (*(self as *const _ as *mut ComPtr<dyn IUnknown>))
                .query_interface(&S::IID as *const IID, &mut ppv)
        };
        if failed(hr) {
            assert!(hr == E_NOINTERFACE);
            return None;
        }
        unsafe { Some(ComPtr::new(ppv)) }
    }
}

impl<T: ComInterface + ?Sized> Drop for ComPtr<T> {
    fn drop(&mut self) {
        println!("Dropped!");
        unsafe {
            (*(self as *const _ as *mut ComPtr<dyn IUnknown>)).release();
        }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        let mut new_ptr = ComPtr {
            ptr: self.ptr,
            phantom: PhantomData,
        };
        new_ptr.cast_and_add_ref();
        new_ptr
    }
}
