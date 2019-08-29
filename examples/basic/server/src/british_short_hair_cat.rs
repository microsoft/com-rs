use com::{iunknown_gen_vtable, IUnknown, IUnknownVPtr, IUnknownVTable, IID_IUNKNOWN};
use interface::{
    ianimal::{IAnimal, IAnimalVPtr, IAnimalVTable, IID_IANIMAL},
    ianimal_gen_vtable,
    icat::{ICat, ICatVPtr, ICatVTable, IID_ICAT},
    icat_gen_vtable,
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalVPtr, IDomesticAnimalVTable, IID_IDOMESTIC_ANIMAL,
    },
    idomesticanimal_gen_vtable,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[repr(C)]
pub struct BritishShortHairCat {
    // inner must always be first because Cat is actually an ICat with one extra field at the end
    inner_one: ICatVPtr,
    inner_two: IDomesticAnimalVPtr,
    ref_count: u32,
}

impl Drop for BritishShortHairCat {
    fn drop(&mut self) {
        let _ = unsafe {
            Box::from_raw(self.inner_one as *mut ICatVTable);
            Box::from_raw(self.inner_two as *mut IDomesticAnimalVTable);
        };
    }
}

impl IDomesticAnimal for BritishShortHairCat {
    fn train(&mut self) -> HRESULT {
        println!("Training...");
        NOERROR
    }
}

impl ICat for BritishShortHairCat {
    fn ignore_humans(&mut self) -> HRESULT {
        println!("Ignoring Humans...");
        NOERROR
    }
}

impl IAnimal for BritishShortHairCat {
    fn eat(&mut self) -> HRESULT {
        println!("Eating...");
        NOERROR
    }
}

impl IUnknown for BritishShortHairCat {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;

            if IsEqualGUID(riid, &IID_IUNKNOWN)
                | IsEqualGUID(riid, &IID_ICAT)
                | IsEqualGUID(riid, &IID_IANIMAL)
            {
                *ppv = &self.inner_one as *const _ as *mut c_void;
            } else if IsEqualGUID(riid, &IID_IDOMESTIC_ANIMAL) {
                *ppv = &self.inner_two as *const _ as *mut c_void;
            } else {
                println!("Returning NO INTERFACE.");
                return E_NOINTERFACE;
            }

            println!("Successful!.");
            self.add_ref();
            NOERROR
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
            println!("Count is 0 for BritishShortHairCat. Freeing memory...");
            drop(self)
        }
        count
    }
}

impl BritishShortHairCat {
    pub(crate) fn new() -> BritishShortHairCat {
        println!("Allocating new Vtable...");
        let icat_vtable = icat_gen_vtable!(BritishShortHairCat, 0);
        let icat_vptr = Box::into_raw(Box::new(icat_vtable));
        let idomesticanimal_vtable = idomesticanimal_gen_vtable!(BritishShortHairCat, 1);
        let idomesticanimal_vptr = Box::into_raw(Box::new(idomesticanimal_vtable));

        BritishShortHairCat {
            inner_one: icat_vptr,
            inner_two: idomesticanimal_vptr,
            ref_count: 0,
        }
    }
}
