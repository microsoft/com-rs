use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IID, REFCLSID, REFIID},
        minwindef::LPVOID,
        winerror::{HRESULT, S_FALSE, S_OK},
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

pub struct ApartmentThreadedRuntime {
    _not_send: *const (),
}

impl ApartmentThreadedRuntime {
    pub fn new() -> Result<ApartmentThreadedRuntime, HRESULT> {
        // need to initialize `runtime` before calling `CoInitializeEx`, see below
        let runtime = ApartmentThreadedRuntime {
            _not_send: std::ptr::null(),
        };
        let hr =
            unsafe { CoInitializeEx(std::ptr::null_mut::<c_void>(), COINIT_APARTMENTTHREADED) };
        // if hr is S_OK all is good, if it is S_FALSE, then the runtime was already initialized
        if hr != S_OK || hr != S_FALSE {
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
        unsafe {
            Ok(ComPtr::new(
                self.create_raw_instance::<T>(clsid)? as *mut c_void
            ))
        }
    }

    pub fn create_raw_instance<T: ComInterface + ?Sized>(
        &self,
        clsid: &IID,
    ) -> Result<*mut T::VPtr, HRESULT> {
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

        Ok(instance as *mut T::VPtr)
    }
}

impl std::ops::Drop for ApartmentThreadedRuntime {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
