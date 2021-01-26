//! Everything related to the [IUnknown](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown) COM interface

use crate::sys::{E_NOINTERFACE, E_POINTER, FAILED};
use crate::sys::{GUID, HRESULT};
use crate::{interfaces, Interface, IID};

use core::ffi::c_void;

interfaces! {
    /// [IUnknown](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown) COM interface
    #[uuid("00000000-0000-0000-C000-000000000046")]
    pub unsafe interface IUnknown {
        /// The COM [`QueryInterface` Method]
        ///
        /// This method normally should not be called directly. Interfaces that implement
        /// `IUnknown` also implement [`IUnknown::query_interface`] which is a safe wrapper around
        /// `IUnknown::QueryInterface`.
        ///
        /// [`QueryInterface` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-queryinterface(refiid_void)
        /// [`IUnknown::query_interface`]: trait.IUnknown.html#method.query_interface
        pub unsafe fn QueryInterface(&self, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT;

        /// The COM [`AddRef` Method]
        ///
        /// This method normally should not be called directly. This method is used by
        /// the `Clone` implementation of interfaces to implement the reference counting mechanism.
        ///
        /// [`AddRef` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-addref
        pub unsafe fn AddRef(&self) -> u32;

        /// The COM [`Release` Method]
        ///
        /// This method normally should not be called directly. This method is used by
        /// the `Drop` implementation of interfaces to implement the reference counting mechanism.
        ///
        /// [`Release` Method]: https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-release
        /// [`ComPtr`]: struct.ComPtr.html
        pub unsafe fn Release(&self) -> u32;
    }

}

impl IUnknown {
    /// A safe version of `QueryInterface`.
    ///
    /// If the backing class implements the interface `I` then a `Some`
    /// containing an `ComPtr` pointing to that interface will be returned
    /// otherwise `None` will be returned.
    pub fn query_interface<I: Interface>(&self) -> Option<I> {
        let mut ppv = None;
        let hr = unsafe {
            self.QueryInterface(
                &I::IID as *const IID,
                &mut ppv as *mut _ as *mut *mut c_void,
            )
        };
        if FAILED(hr) {
            assert!(
                hr == E_NOINTERFACE || hr == E_POINTER,
                "QueryInterface returned non-standard error"
            );
            return None;
        }
        debug_assert!(ppv.is_some());
        ppv
    }
}
