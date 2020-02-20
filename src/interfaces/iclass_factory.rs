use crate::com_interface;
use crate::sys::{BOOL, FAILED, HRESULT, IID};
use std::ffi::c_void;

use crate::{
    interfaces::iunknown::{IUnknown, IUnknownVPtr},
    ComInterface, ComRc,
};

#[com_interface("00000001-0000-0000-c000-000000000046")]
pub trait IClassFactory: IUnknown {
    unsafe fn create_instance(
        &self,
        aggr: *mut IUnknownVPtr,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    unsafe fn lock_server(&self, increment: BOOL) -> HRESULT;
}

impl ComRc<dyn IClassFactory> {
    pub fn get_instance<T: ComInterface + ?Sized>(&self) -> Option<ComRc<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let aggr = std::ptr::null_mut();
        let hr = unsafe { self.create_instance(aggr, &T::IID as *const IID, &mut ppv) };
        if FAILED(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(unsafe { ComRc::from_raw(ppv as *mut _) })
    }
}
