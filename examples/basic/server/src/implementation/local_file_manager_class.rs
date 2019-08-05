use std::os::raw::c_void;

use crate::implementation::LocalFileManager;
use com::{
    IClassFactory, IClassFactoryMethods, IClassFactoryVTable, IID_IUnknown, IUnknownMethods,
    RawIClassFactory, RawIUnknown, BOOL, E_INVALIDARG, E_NOINTERFACE, HRESULT, IID,
    IID_ICLASS_FACTORY, NOERROR, S_OK,
};

#[repr(C)]
pub struct LocalFileManagerClass {
    inner: IClassFactory,
    ref_count: u32,
}

impl Drop for LocalFileManagerClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner.inner.vtable as *mut IClassFactoryVTable) };
    }
}

unsafe extern "stdcall" fn query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface on LocalFileManagerClass...");
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
    let this = this as *mut LocalFileManagerClass;
    (*this).ref_count += 1;
    println!("LFMC Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this as *mut LocalFileManagerClass;
    (*this).ref_count -= 1;
    println!("LFMC Count now {}", (*this).ref_count);
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
    if !aggregate.is_null() && *riid != IID_IUnknown {
        *ppv = std::ptr::null_mut::<c_void>();
        return E_INVALIDARG;
    }

    let lfm = Box::into_raw(Box::new(LocalFileManager::new(aggregate)));

    // This check has to be here because it can only be done after object
    // is allocated on the heap (address of nonDelegatingUnknown fixed)
    if aggregate.is_null() {
        (*lfm).iunk_to_use = &((*lfm).non_delegating_unk) as *const _ as *mut RawIUnknown;
    }

    // As an aggregable object, we have to add_ref through the
    // non-delegating IUnknown on creation. Otherwise, we might
    // add_ref the outer object if aggregated.
    ((*lfm).non_delegating_unk).raw_add_ref();
    let hr = (*lfm).non_delegating_unk.raw_query_interface(riid, ppv);
    ((*lfm).non_delegating_unk).raw_release();
    hr
}

unsafe extern "stdcall" fn lock_server(increment: BOOL) -> HRESULT {
    println!("LockServer called");
    S_OK
}

impl LocalFileManagerClass {
    pub(crate) fn new() -> LocalFileManagerClass {
        println!("Allocating new Vtable for LocalFileManagerClass...");
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
        LocalFileManagerClass {
            inner: IClassFactory { inner },
            ref_count: 0,
        }
    }
}
