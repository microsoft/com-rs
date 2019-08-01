use std::os::raw::c_void;

use crate::interface::{
    ianimal::{IAnimalMethods, RawIAnimal, IID_IANIMAL},
    icat::{ICat, ICatMethods, ICatVTable, RawICat, IID_ICAT},
};
use common::{IID_IUnknown, IUnknownMethods, RawIUnknown, E_NOINTERFACE, HRESULT, IID, NOERROR};

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[repr(C)]
pub struct BritishShortHairCat {
    // inner must always be first because Cat is actually an ICat with one extra field at the end
    inner: ICat,
    ref_count: u32,
}

impl Drop for BritishShortHairCat {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner.inner.vtable as *mut ICatVTable) };
    }
}

unsafe extern "stdcall" fn query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface...");
    if *riid == IID_IUnknown || *riid == IID_ICAT || *riid == IID_IANIMAL {
        *ppv = this as *mut c_void;
        (*this).raw_add_ref();
        NOERROR
    } else {
        E_NOINTERFACE
    }
}

unsafe extern "stdcall" fn add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this as *mut BritishShortHairCat;
    (*this).ref_count += 1;
    println!("Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this as *mut BritishShortHairCat;
    (*this).ref_count -= 1;
    println!("Count now {}", (*this).ref_count);
    let count = (*this).ref_count;
    if count == 0 {
        println!("Count is 0. Freeing memory...");
        let _ = Box::from_raw(this);
    }
    count
}

unsafe extern "stdcall" fn ignore_humans(_this: *mut RawICat) -> HRESULT {
    println!("Ignoring...");
    NOERROR
}

unsafe extern "stdcall" fn eat(_this: *mut RawIAnimal) -> HRESULT {
    println!("Eating...");
    NOERROR
}

impl BritishShortHairCat {
    pub(crate) fn new() -> BritishShortHairCat {
        println!("Allocating new Vtable...");
        let iunknown = IUnknownMethods {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let ianimal = IAnimalMethods { Eat: eat };
        let icat = ICatMethods {
            IgnoreHumans: ignore_humans,
        };
        let vtable = Box::into_raw(Box::new(ICatVTable(iunknown, ianimal, icat)));
        let inner = RawICat { vtable };
        BritishShortHairCat {
            inner: ICat { inner },
            ref_count: 0,
        }
    }
}
