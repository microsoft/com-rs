use crate::british_short_hair_cat::BritishShortHairCat;
use com::{
    IClassFactory, IClassFactoryVPtr, IUnknown, IUnknownVPtr, IID_ICLASS_FACTORY, IID_IUNKNOWN,
};
use interface::icat_class::{ICatClassVTable, IID_ICAT_CLASS};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID},
        minwindef::BOOL,
        winerror::{CLASS_E_NOAGGREGATION, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
    },
};

#[repr(C)]
pub struct BritishShortHairCatClass {
    inner: IClassFactoryVPtr,
    ref_count: u32,
}

impl IClassFactory for BritishShortHairCatClass {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        println!("Creating instance...");
        if !aggr.is_null() {
            return CLASS_E_NOAGGREGATION;
        }

        let mut cat = Box::new(BritishShortHairCat::new());
        cat.add_ref();
        let hr = cat.query_interface(riid, ppv);
        cat.release();
        Box::into_raw(cat);

        hr
    }

    fn lock_server(&mut self, _increment: BOOL) -> HRESULT {
        println!("LockServer called");
        S_OK
    }
}

impl IUnknown for BritishShortHairCatClass {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;
            if IsEqualGUID(riid, &IID_IUNKNOWN)
                || IsEqualGUID(riid, &IID_ICLASS_FACTORY)
                || IsEqualGUID(riid, &IID_ICAT_CLASS)
            {
                *ppv = self as *const _ as *mut c_void;
                self.add_ref();
                NOERROR
            } else {
                E_NOINTERFACE
            }
        }
    }

    fn add_ref(&mut self) -> u32 {
        self.ref_count += 1;
        println!("Count now {}", self.ref_count);
        self.ref_count
    }

    fn release(&mut self) -> u32 {
        self.ref_count -= 1;
        println!("Count now {}", self.ref_count);
        let count = self.ref_count;
        if count == 0 {
            println!("Count is 0 for BritishShortHairCatClass. Freeing memory...");
            drop(self);
        }
        count
    }
}

impl Drop for BritishShortHairCatClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner as *mut ICatClassVTable) };
    }
}

impl BritishShortHairCatClass {
    pub(crate) fn new() -> BritishShortHairCatClass {
        println!("Allocating new vtable for CatClass...");
        let icat_class_vtable =
            <dyn IClassFactory as com::Foo<BritishShortHairCatClass>>::vtable::<com::Zero>();
        let vptr = Box::into_raw(Box::new(icat_class_vtable));

        BritishShortHairCatClass {
            inner: vptr,
            ref_count: 0,
        }
    }
}
