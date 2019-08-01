use std::os::raw::c_void;

use crate::implementation::BritishShortHairCat;
use crate::interface::icat_class::{ICatClass, ICatClassVTable, RawICatClass, IID_ICAT_CLASS};
use common::{
    IClassFactoryVTable, IID_IUnknown, IUnknownVTable, RawIUnknown, BOOL, CLASS_E_NOAGGREGATION,
    E_NOINTERFACE, HRESULT, IID, IID_ICLASS_FACTORY, NOERROR, S_OK,
};

#[repr(C)]
pub struct BritishShortHairCatClass {
    inner: ICatClass,
    ref_count: u32,
}

impl Drop for BritishShortHairCatClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner.inner.vtable as *mut ICatClassVTable) };
    }
}

unsafe extern "stdcall" fn query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface on CatClass...");
    if *riid == IID_IUnknown || *riid == IID_ICLASS_FACTORY || *riid == IID_ICAT_CLASS {
        *ppv = this as *mut c_void;
        (*this).raw_add_ref();
        NOERROR
    } else {
        E_NOINTERFACE
    }
}

unsafe extern "stdcall" fn add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this as *mut BritishShortHairCatClass;
    (*this).ref_count += 1;
    println!("Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this as *mut BritishShortHairCatClass;
    (*this).ref_count -= 1;
    println!("Count now {}", (*this).ref_count);
    let count = (*this).ref_count;
    if count == 0 {
        println!("Count is 0. Freeing memory...");
        let _ = Box::from_raw(this);
    }
    count
}

unsafe extern "stdcall" fn create_instance(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Creating instance...");
    if this != std::ptr::null_mut() {
        return CLASS_E_NOAGGREGATION;
    }

    let cat = Box::into_raw(Box::new(BritishShortHairCat::new()));
    (*(cat as *mut RawIUnknown)).raw_add_ref();
    let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
    (*(cat as *mut RawIUnknown)).raw_release();
    hr
}

unsafe extern "stdcall" fn lock_server(increment: BOOL) -> HRESULT {
    println!("LockServer called");
    S_OK
}

impl BritishShortHairCatClass {
    pub(crate) fn new() -> BritishShortHairCatClass {
        println!("Allocating new Vtable for CatClass...");
        let iunknown = IUnknownVTable {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let iclassfactory = IClassFactoryVTable {
            iunknown,
            CreateInstance: create_instance,
            LockServer: lock_server,
        };
        let vtable = Box::into_raw(Box::new(ICatClassVTable {
            iclassfactory,
        }));
        let inner = RawICatClass { vtable };
        BritishShortHairCatClass {
            inner: ICatClass { inner },
            ref_count: 0,
        }
    }
}
