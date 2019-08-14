use crate::WindowsFileManager;
use com::{
    failed, IClassFactory, IClassFactoryMethods, IClassFactoryVTable, IUnknownMethods,
    RawIClassFactory, RawIUnknown, IID_ICLASS_FACTORY, IID_IUNKNOWN,
};
use interface::CLSID_LOCAL_FILE_MANAGER_CLASS;

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFCLSID, REFIID},
        minwindef::{BOOL, LPVOID},
        winerror::{CLASS_E_NOAGGREGATION, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::combaseapi::CoCreateInstance,
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

    let riid_ref = &*riid;
    if IsEqualGUID(riid_ref, &IID_IUNKNOWN) | IsEqualGUID(riid_ref, &IID_ICLASS_FACTORY) {
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
    _this: *mut RawIClassFactory,
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
    let mut unknown_file_manager = std::ptr::null_mut::<c_void>();
    let hr = CoCreateInstance(
        &CLSID_LOCAL_FILE_MANAGER_CLASS as REFCLSID,
        wfm as winapi::um::unknwnbase::LPUNKNOWN,
        CLSCTX_INPROC_SERVER,
        &IID_IUNKNOWN as REFIID,
        &mut unknown_file_manager as *mut LPVOID,
    );
    if failed(hr) {
        println!("Failed to instantiate aggregate! Error: {:x}", hr as u32);
        panic!();
    }
    (*wfm).p_unk_local_file_manager = unknown_file_manager as *mut RawIUnknown;

    // Start reference count only after aggregation
    (*(wfm as *mut RawIUnknown)).raw_add_ref();
    let hr = (*(wfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
    (*(wfm as *mut RawIUnknown)).raw_release();
    hr
}

unsafe extern "stdcall" fn lock_server(_increment: BOOL) -> HRESULT {
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
