use super::{ComInterface, ComPtr};

/// An param for a COM interface method
///
/// This trait should not be manually implemented. This is used
/// as a convenience for generated code.
pub trait ComInterfaceParam<T> {
    /// Convert an item into an param for an COM interface method.
    ///
    /// # Safety
    /// This conversion must only be valid to use for a call across
    /// FFI boundaries. In concrete terms, T is only guranteed to be
    /// in a valid state for as long as `self` is alive.
    unsafe fn into(self) -> T;
}

impl<T> ComInterfaceParam<T> for T {
    unsafe fn into(self) -> T {
        self
    }
}

impl<T> ComInterfaceParam<*const T> for &T {
    unsafe fn into(self) -> *const T {
        self
    }
}

impl<T: ComInterface> ComInterfaceParam<T> for &T {
    unsafe fn into(self) -> T {
        self.alias()
    }
}

impl<T> ComInterfaceParam<*const T> for *mut T {
    unsafe fn into(self) -> *const T {
        self
    }
}

impl<T> ComInterfaceParam<*mut T> for &mut T {
    unsafe fn into(self) -> *mut T {
        self
    }
}

impl<T: ComInterface> ComInterfaceParam<T> for &ComPtr<T> {
    unsafe fn into(self) -> T {
        self.get().alias()
    }
}

impl<T: ComInterface> ComInterfaceParam<*mut T> for &mut ComPtr<T> {
    unsafe fn into(self) -> *mut T {
        self as *mut ComPtr<T> as *mut T
    }
}

impl<T: ComInterface> ComInterfaceParam<*mut Option<T>> for &mut Option<ComPtr<T>> {
    unsafe fn into(self) -> *mut Option<T> {
        self as *mut Option<ComPtr<T>> as *mut Option<T>
    }
}

impl<T: ComInterface> ComInterfaceParam<Option<T>> for &Option<ComPtr<T>> {
    unsafe fn into(self) -> Option<T> {
        self.as_ref().map(|s| s.get().alias())
    }
}

impl<T: ComInterface> ComInterfaceParam<*const T> for &mut ComPtr<T> {
    unsafe fn into(self) -> *const T {
        self as *const ComPtr<T> as *const T
    }
}

impl<T: ComInterface> ComInterfaceParam<*const Option<T>> for &mut Option<ComPtr<T>> {
    unsafe fn into(self) -> *const Option<T> {
        self as *const Option<ComPtr<T>> as *const Option<T>
    }
}
