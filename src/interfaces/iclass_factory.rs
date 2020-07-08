//! Everything related to the [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
use crate::com_interface;
use crate::sys::{BOOL, FAILED, GUID, HRESULT};
use std::ffi::c_void;

use crate::{
    interfaces::iunknown::{IUnknown, IUnknownVPtr},
    ComInterface, ComRc,
};

com_interface! {
    /// [IClassFactory](https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iclassfactory) COM interface
    #[uuid("00000001-0000-0000-c000-000000000046")]
    pub unsafe interface IClassFactory: IUnknown {
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
}

impl ComRc<IClassFactory> {
    /// Get an instance of the associated Co Class
    pub fn get_instance<T: ComInterface>(&self) -> Option<ComRc<T>> {
        let mut ppv = None;
        let hr = unsafe {
            self.create_instance(
                std::ptr::null_mut(),
                &T::IID as *const GUID,
                &mut ppv as *mut _ as _,
            )
        };
        if FAILED(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(ComRc::new(ppv.unwrap()))
    }
}
