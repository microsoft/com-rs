use crate::ComInterface;

use std::ptr::NonNull;

use std::marker::PhantomData;
use winapi::ctypes::c_void;

/// A transparent ptr to a COM interface.
///
/// This is normally _not_ the correct way to interact with an interface. Normally
/// you'll want to to interact with an interface through a [`InterfaceRc`] which
/// automatically calls `AddRef` and `Release` at the right time .
///
/// [`InterfaceRc`]: struct.InterfaceRc.html
pub struct InterfacePtr<T: ?Sized + ComInterface> {
    ptr: NonNull<c_void>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized + ComInterface> InterfacePtr<T> {
    /// Creates a new `InterfacePtr` that comforms to the interface T
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid interface pointer for interface `T`. An interface
    /// pointer as the name suggests points to an interface struct. A valid
    /// interface is itself trivial castable to a `*mut T::VTable`. In other words,
    /// `ptr` should also be equal to `*mut *mut T::VTable`
    ///
    /// `ptr` must live for at least as long as the `InterfacePtr`
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null
    pub unsafe fn new(ptr: *mut c_void) -> InterfacePtr<T> {
        InterfacePtr {
            ptr: NonNull::new(ptr).expect("ptr was null"),
            phantom: PhantomData,
        }
    }

    /// Gets the underlying interface ptr. This ptr is only guarnteed to live for
    /// as long as the current `InterfacePtr` is alive.
    pub fn as_raw(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }
}
