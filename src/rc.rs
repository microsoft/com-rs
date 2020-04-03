use crate::{interfaces::IUnknown, ComInterface, ComPtr};

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
pub struct ComRc<T: ComInterface + ?Sized> {
    ptr: ComPtr<T>,
}

impl<T: ComInterface + ?Sized> ComRc<T> {
    /// Creates a new `ComRc` that comforms to the interface T.
    pub fn new(ptr: ComPtr<T>) -> ComRc<T> {
        ComRc { ptr }
    }

    /// Construct an `ComRc` from a raw pointer to a COM interface.
    ///
    /// # Safety
    ///
    /// The same safety guarantees as `ComPtr::new` must be upheld by the function.
    pub unsafe fn from_raw(ptr: *mut *mut <T as ComInterface>::VTable) -> Self {
        Self::new(ComPtr::new(ptr))
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `ComRc` is alive.
    pub fn as_raw(&self) -> *mut *mut <T as ComInterface>::VTable {
        self.ptr.as_raw()
    }

    /// A safe version of `QueryInterface`. If the backing CoClass implements the
    /// interface `I` then a `Some` containing an `ComRc` pointing to that
    /// interface will be returned otherwise `None` will be returned.
    pub fn get_interface<I: ComInterface + ?Sized>(&self) -> Option<ComRc<I>> {
        self.ptr.get_interface().map(|ptr| ptr.upgrade())
    }
}

impl<T: ComInterface + ?Sized> Drop for ComRc<T> {
    fn drop(&mut self) {
        unsafe {
            self.release();
            // TODO: Safety issue. self.ptr may contain a dangling pointer at this point
        }
    }
}

impl<T: ComInterface + ?Sized> Clone for ComRc<T> {
    fn clone(&self) -> Self {
        self.ptr.clone().upgrade()
    }
}
