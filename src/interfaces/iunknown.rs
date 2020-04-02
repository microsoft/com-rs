//! Everything related to the [IUnknown](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown) COM interface

use crate::com_interface;
use crate::sys::{GUID, HRESULT};
use std::ffi::c_void;

/// [IUnknown](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown) COM interface
#[com_interface("00000000-0000-0000-C000-000000000046")]
pub trait IUnknown {
    /// The COM [`QueryInterface` Method]
    ///
    /// This method normally should not be called directly. Interfaces that implement
    /// `IUnknown` also implement [`IUnknown::get_interface`] which is a safe wrapper around
    /// `IUnknown::query_interface`.
    ///
    /// [`QueryInterface` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-queryinterface(refiid_void)
    /// [`IUnknown::get_interface`]: trait.IUnknown.html#method.get_interface
    unsafe fn query_interface(&self, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT;

    /// The COM [`AddRef` Method]
    ///
    /// This method normally should not be called directly. This method is used by
    /// [`ComRc`] to implement the reference counting mechanism.
    ///
    /// [`AddRef` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-addref
    /// [`ComRc`]: struct.ComRc.html
    unsafe fn add_ref(&self) -> u32;

    /// The COM [`Release` Method]
    ///
    /// This method normally should not be called directly. This method is used by
    /// [`ComRc`] to implement the reference counting mechanism.
    ///
    /// [`Release` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-release
    /// [`ComRc`]: struct.ComRc.html
    unsafe fn release(&self) -> u32;
}
