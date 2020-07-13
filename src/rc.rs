use crate::ComInterface;
use std::ops::{Deref, DerefMut};

/// A reference counted COM interface.
///
/// This smart pointer type automatically calls `AddRef` when cloned
/// and `Release` when dropped.
///
/// This is normally the correct way to interact with an interface. If for some
/// (usually unsafe) reason, you need to interact with an interface without
/// automatically performing `AddRef` and `Release`, you can use the [`ComPtr`]
/// type.
///
/// [`ComPtr`]: struct.ComPtr.html
#[repr(transparent)]
#[derive(Debug)]
pub struct ComRc<T: ComInterface> {
    ptr: T,
}

impl<T: ComInterface> ComRc<T> {
    /// Creates a new `ComRc` that comforms to the interface T.
    pub fn new(ptr: T) -> ComRc<T> {
        ComRc { ptr }
    }

    /// Construct an `ComRc` from a raw pointer to a COM interface.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid interface pointer for interface `T`. An interface
    /// pointer as the name suggests points to an interface struct. A valid
    /// interface is itself trivial castable to a `*mut T::VTable`. In other words,
    /// `ptr` should also be equal to `*mut *const T::VTable`
    ///
    /// `ptr` must live for at least as long as the `ComPtr`. The underlying
    /// COM interface is assumed to correctly implement AddRef and Release such that
    /// the interface will be valid as long as AddRef has been called more times than
    /// Release.
    ///
    /// AddRef must have been called on the underlying COM interface that `ptr` is pointing
    /// to such that the reference count must be at least 1. It is expected that Release
    /// will eventually be called on this pointer either manually or by the wrapper
    /// being dropped.
    ///
    /// When this struct is dropped, `release` will be called on the underlying interface.
    pub unsafe fn from_raw(ptr: std::ptr::NonNull<*const <T as ComInterface>::VTable>) -> Self {
        // SAFETY: ComInterfaces are required to be aliases to *mut *const <T as ComInterface>::VTable
        Self::new(T::from_raw(ptr))
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `ComRc` is alive.
    pub fn as_raw(&self) -> std::ptr::NonNull<*const <T as ComInterface>::VTable> {
        self.ptr.as_raw()
    }

    /// A safe version of `QueryInterface`. If the backing CoClass implements the
    /// interface `I` then a `Some` containing an `ComRc` pointing to that
    /// interface will be returned otherwise `None` will be returned.
    pub fn get_interface<I: ComInterface>(&self) -> Option<ComRc<I>> {
        self.as_iunknown()
            .get_interface::<I>()
            .map(|ptr| ptr.upgrade())
    }
}

impl<T: ComInterface> Deref for ComRc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
impl<T: ComInterface> DerefMut for ComRc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

impl<T: ComInterface> Drop for ComRc<T> {
    fn drop(&mut self) {
        unsafe {
            self.as_iunknown().release();
        }
    }
}

impl<T: ComInterface> Clone for ComRc<T> {
    fn clone(&self) -> Self {
        unsafe { self.ptr.as_iunknown().add_ref() };
        let raw = self.as_raw();
        Self {
            ptr: unsafe { T::from_raw(raw) },
        }
    }
}
