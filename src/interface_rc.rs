use crate::{interface_ptr::InterfacePtr, interfaces::iunknown::IUnknown, ComInterface, IID};

use std::marker::PhantomData;

use winapi::ctypes::c_void;
use winapi::shared::winerror::{E_NOINTERFACE, E_POINTER, FAILED};

/// A reference counted COM interface. This smart pointer type automatically
/// calls `AddRef` when cloned and `Release` when dropped.
///
/// This is normally the correct way to interact with an interface. If for some
/// (usually unsafe) reason, you need to interact with an interface without
/// automatically performing `AddRef` and `Release`, you can use the [`InterfacePtr`]
/// type.
///
/// [`InterfacePtr`]: struct.InterfacePtr.html
pub struct InterfaceRc<T: ?Sized + ComInterface> {
    ptr: InterfacePtr<T>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized + ComInterface> InterfaceRc<T> {
    /// Creates a new `InterfaceRc` that comforms to the interface T.
    pub fn new(ptr: InterfacePtr<T>) -> InterfaceRc<T> {
        InterfaceRc {
            ptr,
            phantom: PhantomData,
        }
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `InterfaceRc` is alive.
    pub fn as_raw(&self) -> *mut c_void {
        self.ptr.as_raw()
    }

    /// A safe version of `QueryInterface`. If the backing CoClass implements the
    /// interface `I` then a `Some` containing an `InterfaceRc` pointing to that
    /// interface will be returned otherwise `None` will be returned.
    pub fn get_interface<I: ComInterface + ?Sized>(&self) -> Option<InterfaceRc<I>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let hr = unsafe { self.query_interface(&I::IID as *const IID, &mut ppv) };
        if FAILED(hr) {
            assert!(
                hr == E_NOINTERFACE || hr == E_POINTER,
                "QueryInterface returned non-standard error"
            );
            return None;
        }
        assert!(!ppv.is_null(), "The pointer to the interface returned from a successful call to QueryInterface was null");
        Some(InterfaceRc::new(unsafe { InterfacePtr::new(ppv) }))
    }
}

impl<T: ComInterface + ?Sized> Drop for InterfaceRc<T> {
    fn drop(&mut self) {
        unsafe {
            self.release();
            // self.ptr may contain a dangling pointer at this point
        }
    }
}

impl<T: ComInterface> Clone for InterfaceRc<T> {
    fn clone(&self) -> Self {
        let new_ptr = InterfaceRc {
            ptr: self.ptr.clone(),
            phantom: PhantomData,
        };
        new_ptr.add_ref();
        new_ptr
    }
}
