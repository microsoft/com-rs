use crate::com_interface;
use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IID, REFIID},
        minwindef::BOOL,
        ntdef::HRESULT,
        winerror::FAILED,
    },
};

use crate::{
    interfaces::iunknown::{IUnknown, IUnknownVPtr},
    ComInterface, InterfacePtr, InterfaceRc,
};

#[com_interface("00000001-0000-0000-c000-000000000046")]
pub trait IClassFactory: IUnknown {
    unsafe fn create_instance(
        &self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    unsafe fn lock_server(&self, increment: BOOL) -> HRESULT;
}

impl InterfaceRc<dyn IClassFactory> {
    pub fn get_instance<T: ComInterface + ?Sized>(&self) -> Option<InterfaceRc<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let aggr = std::ptr::null_mut();
        let hr = unsafe { self.create_instance(aggr, &T::IID as *const IID, &mut ppv) };
        if FAILED(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(InterfaceRc::new(unsafe { InterfacePtr::new(ppv) }))
    }
}
