//! Everything related to the [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
use crate::interfaces;
use crate::sys::{BOOL, FAILED, GUID, HRESULT};
use core::ffi::c_void;

use crate::{interfaces::iunknown::IUnknown, Interface};

interfaces! {
    /// [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
    #[uuid("00000001-0000-0000-c000-000000000046")]
    pub unsafe interface IClassFactory: IUnknown {
        /// the [CreateInstance](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iclassfactory-createinstance) COM method
        pub unsafe fn CreateInstance(
            &self,
            aggr: Option<IUnknown>,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> HRESULT;
        /// the [LockServer](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iclassfactory-lockserver) COM method
        pub unsafe fn LockServer(&self, increment: BOOL) -> HRESULT;
    }
}

impl IClassFactory {
    /// Create an instance of the associated class
    ///
    /// This is a safe wrapper around `CreateInstance`
    pub fn create_instance<T: Interface>(&self) -> Option<T> {
        let mut ppv = None;
        let hr =
            unsafe { self.CreateInstance(None, &T::IID, &mut ppv as *mut _ as *mut *mut c_void) };
        if FAILED(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(ppv.unwrap())
    }
}
