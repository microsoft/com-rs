#[com_interface(00000001-0000-0000-c000-000000000046)]
pub trait IClassFactory: IUnknown {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    fn lock_server(&mut self, increment: BOOL) -> HRESULT;
}

impl ComPtr<IClassFactory> {
    pub fn get_instance<T: ComInterface + ?Sized>(&mut self) -> Option<ComPtr<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let aggr = std::ptr::null_mut();
        let hr = self.create_instance(aggr, &T::IID as *const IID, &mut ppv);
        if failed(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(ComPtr::new(std::ptr::NonNull::new(ppv as *mut c_void)?))
    }
}

// ----------------------------- DESIRED EXPANSION ------------------------------------------------------
use super::*;
use winapi::ctypes::c_void;
use winapi::shared::guiddef::IID;
use winapi::shared::guiddef::REFIID;
use winapi::shared::minwindef::BOOL;
use winapi::shared::ntdef::HRESULT;

use std::marker::PhantomData;

#[allow(non_upper_case_globals)]
pub const IID_ICLASSFACTORY: IID = IID {
    Data1: 0x01u32,
    Data2: 0u16,
    Data3: 0u16,
    Data4: [0xC0, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0x46u8],
};

#[repr(C)]
pub struct IClassFactoryVTable {
    pub base: <IUnknown as ComInterface>::VTable,
    pub CreateInstance: unsafe extern "stdcall" fn(
        *mut IClassFactoryVPtr,
        *mut IUnknownVPtr,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub LockServer: unsafe extern "stdcall" fn(*mut IClassFactoryVPtr, BOOL) -> HRESULT,
}
pub type IClassFactoryVPtr = *const IClassFactoryVTable;

pub trait IClassFactory: IUnknown {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    fn lock_server(&mut self, increment: BOOL) -> HRESULT;
}

impl<T: IClassFactory + ComInterface + ?Sized> IClassFactory for ComPtr<T> {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut IClassFactoryVPtr;
        unsafe { ((**itf_ptr).CreateInstance)(itf_ptr, aggr, riid, ppv) }
    }

    fn lock_server(&mut self, increment: BOOL) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut IClassFactoryVPtr;
        unsafe { ((**itf_ptr).LockServer)(itf_ptr, increment) }
    }
}

unsafe impl ComInterface for IClassFactory {
    type VTable = IClassFactoryVTable;
    const IID: IID = IID_ICLASSFACTORY;
}

impl ComPtr<IClassFactory> {
    pub fn get_instance<T: ComInterface + ?Sized>(&mut self) -> Option<ComPtr<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let mut aggr = std::ptr::null_mut();
        let hr = unsafe { self.create_instance(aggr, &T::IID as *const IID, &mut ppv) };
        if failed(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(ComPtr::new(std::ptr::NonNull::new(ppv as *mut c_void)?))
    }
}
