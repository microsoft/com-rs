//! COM runtime facilities
//!
//! This includes initializing and uninitializing the COM runtime as well
//! as creating instances of CoClasses
use crate::sys::{
    CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, FAILED, HRESULT, IID, S_FALSE, S_OK,
};
use std::ffi::c_void;

use crate::{
    interfaces::iclass_factory::{IClassFactory, IID_ICLASS_FACTORY},
    CoClass, ComInterface, ComPtr, ComRc,
};

/// The threading model for the COM runtime
#[repr(u32)]
#[non_exhaustive]
pub enum ThreadingModel {
    /// COINIT_APARTMENTTHREADED
    ApartmentThreaded = COINIT_APARTMENTTHREADED,
}

/// Initialize a new apartment threaded runtime.
///
/// This calls `CoInitializeEx` with `COINIT_APARTMENTTHREADED`
///
/// On success, it is up to the user to call [`uninitialize`] when the COM runtime is no longer need.
/// On error, the user should not call [`uninitialize`].
pub fn new_runtime(threading_model: ThreadingModel) -> Result<(), HRESULT> {
    unsafe {
        match CoInitializeEx(std::ptr::null_mut::<c_void>(), threading_model as u32) {
            // S_OK indicates the runtime was initialized, S_FALSE means it was initialized
            // previously. In both cases we need to invoke `CoUninitialize` later.
            S_OK | S_FALSE => Ok(()),

            // Any other result is considered an error here.
            hr => Err(hr),
        }
    }
}

/// Get the class object with the associated [`IID`]
///
/// Calls `CoGetClassObject` internally
pub fn get_class_object(iid: &IID) -> Result<ComRc<dyn IClassFactory>, HRESULT> {
    let mut class_factory = std::ptr::null_mut::<c_void>();
    let hr = unsafe {
        CoGetClassObject(
            iid as *const IID,
            CLSCTX_INPROC_SERVER,
            std::ptr::null_mut::<c_void>(),
            &IID_ICLASS_FACTORY as *const IID,
            &mut class_factory as *mut *mut c_void,
        )
    };
    if FAILED(hr) {
        return Err(hr);
    }

    Ok(unsafe { ComRc::from_raw(class_factory as *mut *mut _) })
}

/// Create an instance of a CoClass with the associated class id
///
/// Calls `CoCreateInstance` internally
pub fn create_instance<T: ComInterface + ?Sized>(clsid: &IID) -> Result<ComRc<T>, HRESULT> {
    unsafe {
        Ok(ComRc::new(create_raw_instance::<T>(
            clsid,
            std::ptr::null_mut(),
        )?))
    }
}

/// Created an aggreated instance
///
/// Calls `CoCreateInstance` internally
pub fn create_aggregated_instance<T: ComInterface + ?Sized, U: CoClass>(
    clsid: &IID,
    outer: &mut U,
) -> Result<ComPtr<T>, HRESULT> {
    unsafe { create_raw_instance::<T>(clsid, outer as *mut U as *mut c_void) }
}

unsafe fn create_raw_instance<T: ComInterface + ?Sized>(
    clsid: &IID,
    outer: *mut c_void,
) -> Result<ComPtr<T>, HRESULT> {
    let mut instance = std::ptr::null_mut::<c_void>();
    let hr = CoCreateInstance(
        clsid as *const IID,
        outer,
        CLSCTX_INPROC_SERVER,
        &T::IID as *const IID,
        &mut instance as *mut *mut c_void,
    );
    if FAILED(hr) {
        return Err(hr);
    }

    Ok(ComPtr::new(instance as *mut _))
}

/// Uninitialize the COM runtime.
///
/// This should only be called if the COM runtime is already running (usually started through
/// [`new_apartment_threaded_runtime`])
/// https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize
pub unsafe fn uninitialize() {
    CoUninitialize()
}
