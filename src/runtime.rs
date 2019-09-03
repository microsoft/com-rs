use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IID, REFCLSID, REFIID},
        minwindef::LPVOID,
        winerror::HRESULT,
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize},
        objbase::COINIT_APARTMENTTHREADED,
    },
};

use crate::{
    failed,
    iclassfactory::{IClassFactory, IID_ICLASS_FACTORY},
    ComInterface, ComPtr,
};

// TODO: is it safe to send this across threads?
pub struct Runtime;

impl Runtime {
    pub fn new() -> Result<Runtime, HRESULT> {
        let runtime = Runtime;
        let hr =
            unsafe { CoInitializeEx(std::ptr::null_mut::<c_void>(), COINIT_APARTMENTTHREADED) };
        if failed(hr) {
            // `runtime` is dropped here calling `CoUninitialize` which needs to happen no matter if
            // `CoInitializeEx` is successful or not.
            // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize
            return Err(hr);
        }
        Ok(runtime)
    }

    // TODO: accept server options
    pub fn get_class_object(&self, iid: &IID) -> Result<ComPtr<dyn IClassFactory>, HRESULT> {
        let mut class_factory = std::ptr::null_mut::<c_void>();
        let hr = unsafe {
            CoGetClassObject(
                iid as REFCLSID,
                CLSCTX_INPROC_SERVER,
                std::ptr::null_mut::<c_void>(),
                &IID_ICLASS_FACTORY as REFIID,
                &mut class_factory as *mut LPVOID,
            )
        };
        if failed(hr) {
            return Err(hr);
        }

        unsafe { Ok(ComPtr::new(class_factory)) }
    }

    // TODO: accept server options
    pub fn create_instance<T: ComInterface + ?Sized>(
        &self,
        clsid: &IID,
    ) -> Result<ComPtr<T>, HRESULT> {
        let mut instance = std::ptr::null_mut::<c_void>();
        let hr = unsafe {
            CoCreateInstance(
                clsid as REFCLSID,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &T::IID as REFIID,
                &mut instance as *mut LPVOID,
            )
        };
        if failed(hr) {
            return Err(hr);
        }

        unsafe { Ok(ComPtr::new(instance)) }
    }
}

impl std::ops::Drop for Runtime {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
