use super::{ComInterface, ComRc};

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

impl<T: ComInterface> ComInterfaceParam<T> for &ComRc<T> {
    unsafe fn into(self) -> T {
        (**self).alias()
    }
}

impl<T: ComInterface> ComInterfaceParam<*mut T> for &mut ComRc<T> {
    unsafe fn into(self) -> *mut T {
        self as *mut ComRc<T> as *mut T
    }
}

impl<T: ComInterface> ComInterfaceParam<*mut Option<T>> for &mut Option<ComRc<T>> {
    unsafe fn into(self) -> *mut Option<T> {
        self as *mut Option<ComRc<T>> as *mut Option<T>
    }
}

impl<T: ComInterface> ComInterfaceParam<*const T> for &mut ComRc<T> {
    unsafe fn into(self) -> *const T {
        self as *const ComRc<T> as *const T
    }
}

impl<T: ComInterface> ComInterfaceParam<*const Option<T>> for &mut Option<ComRc<T>> {
    unsafe fn into(self) -> *const Option<T> {
        self as *const Option<ComRc<T>> as *const Option<T>
    }
}
