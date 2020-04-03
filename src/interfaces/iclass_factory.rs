//! Everything related to the [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
use crate::com_interface;
use crate::sys::{BOOL, FAILED, GUID, HRESULT};
use std::ffi::c_void;

use crate::{
    interfaces::iunknown::{IUnknown, IUnknownVPtr},
    ComInterface, ComRc,
};

/// [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
#[com_interface("00000001-0000-0000-c000-000000000046")]
pub trait IClassFactory: IUnknown {
    /// the [CreateInstance](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iclassfactory-createinstance) COM method
    unsafe fn create_instance(
        &self,
        aggr: *mut IUnknownVPtr,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    /// the [LockServer](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iclassfactory-lockserver) COM method
    unsafe fn lock_server(&self, increment: BOOL) -> HRESULT;
}

impl ComRc<dyn IClassFactory> {
    /// Get an instance of the associated Co Class
    pub fn get_instance<T: ComInterface + ?Sized>(&self) -> Option<ComRc<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let aggr = std::ptr::null_mut();
        let hr = unsafe { self.create_instance(aggr, &T::IID as *const GUID, &mut ppv) };
        if FAILED(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(unsafe { ComRc::from_raw(ppv as *mut _) })
    }
}
