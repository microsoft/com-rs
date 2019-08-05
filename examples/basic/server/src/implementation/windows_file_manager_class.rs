use std::os::raw::c_void;

use crate::implementation::WindowsFileManager;
use crate::CLSID_LOCAL_FILE_MANAGER_CLASS;
use com::{
    failed, CoCreateInstance, IClassFactory, IClassFactoryMethods, IClassFactoryVTable,
    IID_IUnknown, IUnknownMethods, RawIClassFactory, RawIUnknown, BOOL, CLASS_E_NOAGGREGATION,
    CLSCTX_INPROC_SERVER, E_NOINTERFACE, HRESULT, IID, IID_ICLASS_FACTORY, LPVOID, NOERROR,
    REFCLSID, REFIID, S_OK,
};

#[repr(C)]
pub struct WindowsFileManagerClass {
    inner: IClassFactory,
    ref_count: u32,
}

impl Drop for WindowsFileManagerClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner.inner.vtable as *mut IClassFactoryVTable) };
    }
}

unsafe extern "stdcall" fn query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface on CatClass...");
    if *riid == IID_IUnknown || *riid == IID_ICLASS_FACTORY {
        *ppv = this as *mut c_void;
        (*this).raw_add_ref();
        NOERROR
    } else {
        E_NOINTERFACE
    }
}

unsafe extern "stdcall" fn add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this as *mut WindowsFileManagerClass;
    (*this).ref_count += 1;
    println!("WFMC Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this as *mut WindowsFileManagerClass;
    (*this).ref_count -= 1;
    println!("WFMC Count now {}", (*this).ref_count);
    let count = (*this).ref_count;
    if count == 0 {
        println!("Count is 0. Freeing memory...");
        let _ = Box::from_raw(this);
    }
    count
}

unsafe extern "stdcall" fn create_instance(
    this: *mut RawIClassFactory,
    aggregate: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Creating instance...");
    if aggregate != std::ptr::null_mut() {
        return CLASS_E_NOAGGREGATION;
    }

    let wfm = Box::into_raw(Box::new(WindowsFileManager::new()));

    // Instantiate object to aggregate
    let mut pUnkLocalFileManager = std::ptr::null_mut::<c_void>();
    let hr = CoCreateInstance(
        &CLSID_LOCAL_FILE_MANAGER_CLASS as REFCLSID,
        wfm as *mut RawIUnknown,
        CLSCTX_INPROC_SERVER,
        &IID_IUnknown as REFIID,
        &mut pUnkLocalFileManager as *mut LPVOID,
    );
    if failed(hr) {
        println!("Failed to instantiate aggregate! Error: {:x}", hr as u32);
        panic!();
    }
    (*wfm).pUnkLocalFileManager = pUnkLocalFileManager as *mut RawIUnknown;

    // Start reference count only after aggregation
    (*(wfm as *mut RawIUnknown)).raw_add_ref();
    let hr = (*(wfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
    (*(wfm as *mut RawIUnknown)).raw_release();
    hr
}

unsafe extern "stdcall" fn lock_server(increment: BOOL) -> HRESULT {
    println!("LockServer called");
    S_OK
}

impl WindowsFileManagerClass {
    pub(crate) fn new() -> WindowsFileManagerClass {
        println!("Allocating new Vtable for WindowsFileManagerClass...");
        let iunknown = IUnknownMethods {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let iclassfactory = IClassFactoryMethods {
            CreateInstance: create_instance,
            LockServer: lock_server,
        };
        let vtable = Box::into_raw(Box::new(IClassFactoryVTable(iunknown, iclassfactory)));
        let inner = RawIClassFactory { vtable };
        WindowsFileManagerClass {
            inner: IClassFactory { inner },
            ref_count: 0,
        }
    }
}
