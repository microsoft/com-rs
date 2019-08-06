use std::os::raw::c_void;

use crate::BritishShortHairCat;
use com::{
    IClassFactoryMethods, IUnknownMethods, RawIClassFactory, RawIUnknown, BOOL,
    CLASS_E_NOAGGREGATION, E_NOINTERFACE, HRESULT, IID, IID_ICLASS_FACTORY, IID_IUNKNOWN, NOERROR,
    S_OK,
};
use interface::icat_class::{
    ICatClass, ICatClassMethods, ICatClassVTable, RawICatClass, IID_ICAT_CLASS,
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
    if *riid == IID_IUNKNOWN || *riid == IID_ICLASS_FACTORY || *riid == IID_ICAT_CLASS {
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
    _this: *mut RawIClassFactory,
    aggregate: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Creating instance...");
    if aggregate != std::ptr::null_mut() {
        return CLASS_E_NOAGGREGATION;
    }

    let cat = Box::into_raw(Box::new(BritishShortHairCat::new()));
    (*(cat as *mut RawIUnknown)).raw_add_ref();
    let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
    (*(cat as *mut RawIUnknown)).raw_release();
    hr
}

unsafe extern "stdcall" fn lock_server(_increment: BOOL) -> HRESULT {
    println!("LockServer called");
    S_OK
}

impl BritishShortHairCatClass {
    pub(crate) fn new() -> BritishShortHairCatClass {
        println!("Allocating new Vtable for CatClass...");
        let iunknown = IUnknownMethods {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let iclassfactory = IClassFactoryMethods {
            CreateInstance: create_instance,
            LockServer: lock_server,
        };
        let icatclass = ICatClassMethods {};
        let vtable = Box::into_raw(Box::new(ICatClassVTable(
            iunknown,
            iclassfactory,
            icatclass,
        )));
        let inner = RawICatClass { vtable };
        BritishShortHairCatClass {
            inner: ICatClass { inner },
            ref_count: 0,
        }
    }
}
