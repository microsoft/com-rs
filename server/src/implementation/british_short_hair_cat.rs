use std::mem::size_of;
use std::os::raw::c_void;

use crate::interface::{
    ianimal::{IAnimalMethods, RawIAnimal, IID_IANIMAL},
    icat::{ICat, ICatMethods, ICatVTable, RawICat, IID_ICAT},
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalMethods, IDomesticAnimalVTable, RawIDomesticAnimal,
        IID_IDOMESTIC_ANIMAL,
    },
};
use common::{IID_IUnknown, IUnknownMethods, RawIUnknown, E_NOINTERFACE, HRESULT, IID, NOERROR};

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[repr(C)]
pub struct BritishShortHairCat {
    // inner must always be first because Cat is actually an ICat with one extra field at the end
    inner_one: ICat,
    inner_two: IDomesticAnimal,
    ref_count: u32,
}

impl Drop for BritishShortHairCat {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner_one.inner.vtable as *mut ICatVTable) };
    }
}

unsafe extern "stdcall" fn icat_query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface through ICat's IUnknown...");
    let obj = this as *mut BritishShortHairCat;

    match *riid {
        IID_IUnknown | IID_ICAT | IID_IANIMAL => {
            println!("Returning this.");
            *ppv = this as *mut c_void;
        }
        IID_IDOMESTIC_ANIMAL => {
            println!("Returning this add 1.");
            *ppv = this.add(1) as *mut c_void;
        }
        _ => {
            println!("Returning NO INTERFACE.");
            return E_NOINTERFACE;
        }
    }

    println!("Successful!.");
    (*this).raw_add_ref();
    NOERROR
}

unsafe extern "stdcall" fn icat_add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this as *mut BritishShortHairCat;
    (*this).ref_count += 1;
    println!("Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn icat_release(this: *mut RawIUnknown) -> u32 {
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

// TODO: We need to consider abstracting this messy logic away from the production process.
// Adjustor Thunks
unsafe extern "stdcall" fn idomesticanimal_query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    icat_query_interface(this.sub(1), riid, ppv)
}

unsafe extern "stdcall" fn idomesticanimal_add_ref(this: *mut RawIUnknown) -> u32 {
    icat_add_ref(this.sub(1))
}

unsafe extern "stdcall" fn idomesticanimal_release(this: *mut RawIUnknown) -> u32 {
    icat_release(this.sub(1))
}

unsafe extern "stdcall" fn ignore_humans(_this: *mut RawICat) -> HRESULT {
    println!("Ignoring...");
    NOERROR
}

unsafe extern "stdcall" fn eat(_this: *mut RawIAnimal) -> HRESULT {
    println!("Eating...");
    NOERROR
}

unsafe extern "stdcall" fn train(_this: *mut RawIDomesticAnimal) -> HRESULT {
    println!("Trainig...");
    NOERROR
}

impl BritishShortHairCat {
    pub(crate) fn new() -> BritishShortHairCat {
        println!("Allocating new Vtable...");

        /* Initialising VTable for ICat */
        let icat_iunknown = IUnknownMethods {
            QueryInterface: icat_query_interface,
            Release: icat_release,
            AddRef: icat_add_ref,
        };
        let icat_ianimal = IAnimalMethods { Eat: eat };

        let icat = ICatMethods {
            IgnoreHumans: ignore_humans,
        };
        let icat_vtable = Box::into_raw(Box::new(ICatVTable(icat_iunknown, icat_ianimal, icat)));
        println!("ICat VTable address: {:p}", icat_vtable);
        let icat_inner = RawICat {
            vtable: icat_vtable,
        };

        /* Initialising VTable for IDomesticAnimal */
        /* Initialising VTable for ICat */
        let idomesticanimal_iunknown = IUnknownMethods {
            QueryInterface: idomesticanimal_query_interface,
            Release: idomesticanimal_release,
            AddRef: idomesticanimal_add_ref,
        };
        let idomesticanimal_ianimal = IAnimalMethods { Eat: eat };

        let idomesticanimal = IDomesticAnimalMethods { Train: train };
        let idomesticanimal_vtable = Box::into_raw(Box::new(IDomesticAnimalVTable(
            idomesticanimal_iunknown,
            idomesticanimal_ianimal,
            idomesticanimal,
        )));
        println!(
            "IDomesticAnimal VTable address: {:p}",
            idomesticanimal_vtable
        );
        let idomesticanimal_inner = RawIDomesticAnimal {
            vtable: idomesticanimal_vtable,
        };

        let out = BritishShortHairCat {
            inner_one: ICat { inner: icat_inner },
            inner_two: IDomesticAnimal {
                inner: idomesticanimal_inner,
            },
            ref_count: 0,
        };

        out
    }
}
