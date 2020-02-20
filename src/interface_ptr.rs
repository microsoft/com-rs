use crate::sys::{E_NOINTERFACE, E_POINTER, FAILED};
use crate::{interfaces::IUnknown, ComInterface, ComRc, IID};

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A transparent ptr to a COM interface.
///
/// This is normally _not_ the correct way to interact with an interface. Normally
/// you'll want to to interact with an interface through a [`ComRc`] which
/// automatically calls `AddRef` and `Release` at the right time .
///
/// [`ComRc`]: struct.ComRc.html
#[repr(transparent)]
pub struct ComPtr<T: ComInterface + ?Sized> {
    ptr: NonNull<*mut <T as ComInterface>::VTable>,
    phantom: PhantomData<T>,
}

impl<T: ComInterface + ?Sized> ComPtr<T> {
    /// Creates a new `ComPtr` that comforms to the interface T
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid interface pointer for interface `T`. An interface
    /// pointer as the name suggests points to an interface struct. A valid
    /// interface is itself trivial castable to a `*mut T::VTable`. In other words,
    /// `ptr` should also be equal to `*mut *mut T::VTable`
    ///
    /// `ptr` must live for at least as long as the `ComPtr`. The underlying
    /// COM interface is assumed to correctly implement AddRef and Release such that
    /// the interface will be valid as long as AddRef has been called more times than
    /// Release.
    ///
    /// AddRef must have been called on the underlying COM interface that `ptr` is pointing
    /// to such that the reference count must be at least 1. It is expected that Release
    /// will eventually be called on this pointer either manually or by passing it into
    /// `ComRc::new` which will cause Release to be called on drop of the rc.
    ///
    /// When this struct is dropped, `release` will be called on the underlying interface.
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null
    pub unsafe fn new(ptr: *mut *mut <T as ComInterface>::VTable) -> ComPtr<T> {
        ComPtr {
            ptr: NonNull::new(ptr).expect("ComPtr's ptr was null"),
            phantom: PhantomData,
        }
    }

    /// Upgrade the `ComPtr` to an `ComRc`
    pub fn upgrade(self) -> ComRc<T> {
        ComRc::new(self)
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `ComPtr` is alive.
    pub fn as_raw(&self) -> *mut *mut <T as ComInterface>::VTable {
        self.ptr.as_ptr()
    }

    /// A safe version of `QueryInterface`. If the backing CoClass implements the
    /// interface `I` then a `Some` containing an `ComRc` pointing to that
    /// interface will be returned otherwise `None` will be returned.
    pub fn get_interface<I: ComInterface + ?Sized>(&self) -> Option<ComPtr<I>> {
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
        Some(unsafe { ComPtr::new(ppv as *mut *mut _) })
    }
}

impl<T: ComInterface + ?Sized> std::convert::From<ComRc<T>> for ComPtr<T> {
    /// Convert from an `ComRc` to an `ComPtr`
    ///
    /// Note that this does not call the release on the underlying interface
    /// which gurantees that the ComPtr will still point to a valid
    /// interface. If Release is never called on this pointer, than memory
    /// may be leaked.
    fn from(rc: crate::ComRc<T>) -> Self {
        let result = unsafe { ComPtr::new(rc.as_raw()) };
        // for get the rc so that its drop impl which calls release is not called
        std::mem::forget(rc);
        result
    }
}

impl<T: ComInterface + ?Sized> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.add_ref();
            ComPtr::new(self.ptr.as_ptr())
        }
    }
}
