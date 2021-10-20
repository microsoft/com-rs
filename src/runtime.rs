//! COM runtime facilities
//!
//! This includes initializing the COM runtime as well as creating instances of COM classes
use crate::sys::{
    CoCreateInstance, CoGetClassObject, CoIncrementMTAUsage, CoInitializeEx, CoUninitialize,
    CLSCTX_INPROC_SERVER, CLSID, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED, FAILED, HRESULT,
    IID, S_FALSE, S_OK,
};
use core::ffi::c_void;

use crate::Interface;

/// Initialize a new multithreaded apartment (MTA) runtime. This will ensure
/// that an MTA is running for the process. Every new thread will implicitly
/// be in the MTA unless a different apartment type is chosen (through [`init_apartment`])
///
/// This calls `CoIncrementMTAUsage`
///
/// This function only needs to be called once per process.
pub fn init_runtime() -> Result<(), HRESULT> {
    let mut _cookie = core::ptr::null_mut::<c_void>();
    match unsafe { CoIncrementMTAUsage(&mut _cookie as *mut _ as *mut _) } {
        // S_OK indicates the runtime was initialized
        S_OK => Ok(()),
        // Any other result is considered an error here.
        hr => Err(hr),
    }
}

/// The threading model of the current thread's apartment
#[repr(u32)]
#[non_exhaustive]
pub enum ApartmentType {
    /// A single-threaded apartment (COINIT_APARTMENTTHREADED)
    SingleThreaded = COINIT_APARTMENTTHREADED,
    /// A multi-threaded apartment (COINIT_MULTITHREADED)
    Multithreaded = COINIT_MULTITHREADED,
}

/// Establish an apartment type for the current thread.
///
/// This can only be called once per thread and will return an error if
/// it is called more than once.
///
/// In  general this should only be called on threads created by the user.
///
/// This wraps `CoInitializeEx`. The user is still responsible for establishing
/// a message pump in the case of an STA
// TODO: create a special `spawn` function for spawning a thread
// with a specific apartment type.
// TODO: add helpers for establishing a message pump
pub fn init_apartment(apartment_type: ApartmentType) -> Result<(), HRESULT> {
    match unsafe { CoInitializeEx(core::ptr::null_mut::<c_void>(), apartment_type as u32) } {
        // S_OK indicates the runtime was initialized
        S_OK | S_FALSE => Ok(()),
        // Any other result is considered an error here.
        hr => Err(hr),
    }
}

/// Uninitialize a COM apartment thread.
///
/// This uses `CoUninitialize`
///
/// This should only be called if the user already initialized the thread as a specific apartment type
/// (usually started through [`init_apartment`]).
/// <https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize>
pub fn deinit_apartment() {
    unsafe { CoUninitialize() }
}

/// An apartment runtime configuration.
///
/// This initializes a thread as a certain [`ApartmentType`] and uninitializes on `drop`
pub struct ApartmentRuntime {
    _priv: *const (), // Ensure that this struct is !Send
}

impl ApartmentRuntime {
    /// Initialize the thread as an [`ApartmentType`]
    pub fn new(apartment_type: ApartmentType) -> Result<Self, HRESULT> {
        init_apartment(apartment_type)?;
        Ok(Self {
            _priv: core::ptr::null(),
        })
    }
}

impl Drop for ApartmentRuntime {
    fn drop(&mut self) {
        deinit_apartment()
    }
}

/// Get the class object with the associated [`CLSID`]
///
/// Calls `CoGetClassObject` internally
pub fn get_class_object<T: Interface>(class_id: &CLSID) -> Result<T, HRESULT> {
    let mut class = None;
    let hr = unsafe {
        CoGetClassObject(
            class_id as *const CLSID,
            CLSCTX_INPROC_SERVER,
            core::ptr::null_mut::<c_void>(),
            &T::IID as *const IID,
            &mut class as *mut _ as _,
        )
    };
    if FAILED(hr) {
        return Err(hr);
    }

    Ok(class.unwrap())
}

/// Create an instance of a COM class with the associated class id
///
/// Calls `CoCreateInstance` internally
pub fn create_instance<T: Interface>(class_id: &CLSID) -> Result<T, HRESULT> {
    unsafe { create_raw_instance::<T>(class_id, core::ptr::null_mut()) }
}

/// A helper for creating both regular and aggregated instances
unsafe fn create_raw_instance<T: Interface>(
    class_id: &CLSID,
    outer: *mut c_void,
) -> Result<T, HRESULT> {
    let mut instance = None;
    let hr = CoCreateInstance(
        class_id as *const CLSID,
        outer,
        CLSCTX_INPROC_SERVER,
        &T::IID as *const IID,
        &mut instance as *mut _ as _,
    );
    if FAILED(hr) {
        return Err(hr);
    }

    Ok(instance.unwrap())
}
