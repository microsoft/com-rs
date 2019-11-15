use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IID, REFCLSID, REFIID},
        minwindef::LPVOID,
        winerror::{FAILED, HRESULT, S_FALSE, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize},
        objbase::COINIT_APARTMENTTHREADED,
        unknwnbase::LPUNKNOWN,
    },
};

use crate::{
    interfaces::iclass_factory::{IClassFactory, IID_ICLASS_FACTORY},
    CoClass, ComInterface, InterfacePtr, InterfaceRc,
};

pub struct ApartmentThreadedRuntime {
    _not_send: *const (),
}

impl ApartmentThreadedRuntime {
    pub fn new() -> Result<ApartmentThreadedRuntime, HRESULT> {
        // Attempt to initialize the runtime first. `CoUninitialize` should be called only if this
        // is successful. Since the `CoUninitialize` call is made through the `Drop` implementation
        // of `ApartmentThreadedRuntime`, we need to be careful to not instantiate the runtime in
        // case the `CoInitializeEx` fails.
        //
        // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize
        unsafe {
            match CoInitializeEx(std::ptr::null_mut::<c_void>(), COINIT_APARTMENTTHREADED) {
                // S_OK indicates the runtime was initialized, S_FALSE means it was initialized
                // previously. In both cases we need to invoke `CoUninitialize` later.
                S_OK | S_FALSE => Ok(ApartmentThreadedRuntime {
                    _not_send: std::ptr::null(),
                }),

                // Any other result is considered an error here.
                hr => Err(hr),
            }
        }
    }

    pub fn get_class_object(&self, iid: &IID) -> Result<InterfaceRc<dyn IClassFactory>, HRESULT> {
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
        if FAILED(hr) {
            return Err(hr);
        }

        Ok(InterfaceRc::new(unsafe {
            InterfacePtr::new(class_factory)
        }))
    }

    pub fn create_instance<T: ComInterface + ?Sized>(
        &self,
        clsid: &IID,
    ) -> Result<InterfaceRc<T>, HRESULT> {
        unsafe {
            Ok(InterfaceRc::new(
                self.create_raw_instance::<T>(clsid, std::ptr::null_mut())?,
            ))
        }
    }

    pub fn create_aggregated_instance<T: ComInterface + ?Sized, U: CoClass>(
        &self,
        clsid: &IID,
        outer: &mut U,
    ) -> Result<InterfacePtr<T>, HRESULT> {
        unsafe { self.create_raw_instance::<T>(clsid, outer as *mut U as LPUNKNOWN) }
    }

    pub unsafe fn create_raw_instance<T: ComInterface + ?Sized>(
        &self,
        clsid: &IID,
        outer: LPUNKNOWN,
    ) -> Result<InterfacePtr<T>, HRESULT> {
        let mut instance = std::ptr::null_mut::<c_void>();
        let hr = CoCreateInstance(
            clsid as REFCLSID,
            outer,
            CLSCTX_INPROC_SERVER,
            &T::IID as REFIID,
            &mut instance as *mut LPVOID,
        );
        if FAILED(hr) {
            return Err(hr);
        }

        Ok(InterfacePtr::new(instance))
    }
}

impl std::ops::Drop for ApartmentThreadedRuntime {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
