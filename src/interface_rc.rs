use crate::{interfaces::IUnknown, ComInterface, InterfacePtr};

/// A reference counted COM interface. This smart pointer type automatically
/// calls `AddRef` when cloned and `Release` when dropped.
///
/// This is normally the correct way to interact with an interface. If for some
/// (usually unsafe) reason, you need to interact with an interface without
/// automatically performing `AddRef` and `Release`, you can use the [`InterfacePtr`]
/// type.
///
/// [`InterfacePtr`]: struct.InterfacePtr.html
pub struct InterfaceRc<T: ComInterface + ?Sized> {
    ptr: InterfacePtr<T>,
}

impl<T: ComInterface + ?Sized> InterfaceRc<T> {
    /// Creates a new `InterfaceRc` that comforms to the interface T.
    pub fn new(ptr: InterfacePtr<T>) -> InterfaceRc<T> {
        InterfaceRc { ptr }
    }

    /// Construct an `InterfaceRc` from a raw pointer to a COM interface.
    ///
    /// # Safety
    ///
    /// The same safety guarantees as `InterfacePtr::new` must be upheld by the function.
    pub unsafe fn from_raw(ptr: *mut *mut <T as ComInterface>::VTable) -> Self {
        Self::new(InterfacePtr::new(ptr))
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `InterfaceRc` is alive.
    pub fn as_raw(&self) -> *mut *mut <T as ComInterface>::VTable {
        self.ptr.as_raw()
    }

    /// A safe version of `QueryInterface`. If the backing CoClass implements the
    /// interface `I` then a `Some` containing an `InterfaceRc` pointing to that
    /// interface will be returned otherwise `None` will be returned.
    pub fn get_interface<I: ComInterface + ?Sized>(&self) -> Option<InterfaceRc<I>> {
        self.ptr.get_interface().map(|ptr| ptr.upgrade())
    }
}

impl<T: ComInterface + ?Sized> Drop for InterfaceRc<T> {
    fn drop(&mut self) {
        unsafe {
            self.release();
            // TODO: Safety issue. self.ptr may contain a dangling pointer at this point
        }
    }
}

impl<T: ComInterface + ?Sized> Clone for InterfaceRc<T> {
    fn clone(&self) -> Self {
        self.ptr.clone().upgrade()
    }
}
