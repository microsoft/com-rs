use com::{IUnknownVPtr, IID_IUNKNOWN, IUnknown, IUnknownVTable,};
use interface::{
    ianimal::{IAnimal, IAnimalVPtr, IID_IANIMAL, IAnimalVTable,},
    icat::{ICat, ICatVTable, ICatVPtr, IID_ICAT},
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalVTable, IDomesticAnimalVPtr,
        IID_IDOMESTIC_ANIMAL,
    },
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

// Adjustor Thunks for ICat
unsafe extern "stdcall" fn icat_query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface through ICat's IUnknown...");
    let this = this as *mut BritishShortHairCat;
    (*this).query_interface(riid, ppv)

    // let riid = &*riid;
    // if IsEqualGUID(riid, &IID_IUNKNOWN)
    //     | IsEqualGUID(riid, &IID_ICAT)
    //     | IsEqualGUID(riid, &IID_IANIMAL)
    // {
    //     println!("Returning this.");
    //     *ppv = this as *mut c_void;
    // } else if IsEqualGUID(riid, &IID_IDOMESTIC_ANIMAL) {
    //     println!("Returning this add 1.");
    //     *ppv = this.add(1) as *mut c_void;
    // } else {
    //     println!("Returning NO INTERFACE.");
    //     return E_NOINTERFACE;
    // }

    // println!("Successful!.");
    // (*this).raw_add_ref();
    // NOERROR
}

unsafe extern "stdcall" fn icat_add_ref(this: *mut IUnknownVPtr) -> u32 {
    println!("Adding ref...");
    let this = this as *mut BritishShortHairCat;
    (*this).add_ref()
    // (*this).ref_count += 1;
    // println!("Count now {}", (*this).ref_count);
    // (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn icat_release(this: *mut IUnknownVPtr) -> u32 {
    println!("Releasing...");
    let this = this as *mut BritishShortHairCat;
    (*this).release()
    // (*this).ref_count -= 1;
    // println!("Count now {}", (*this).ref_count);
    // let count = (*this).ref_count;
    // if count == 0 {
    //     println!("Count is 0. Freeing memory...");
    //     let _ = Box::from_raw(this);
    // }
    // count
}

unsafe extern "stdcall" fn icat_ignore_humans(this: *mut ICatVPtr) -> HRESULT {
    let this = this as *mut BritishShortHairCat;
    (*this).ignore_humans()
    // println!("Ignoring...");
    // NOERROR
}

unsafe extern "stdcall" fn icat_eat(this: *mut IAnimalVPtr) -> HRESULT {
    let this = this as *mut BritishShortHairCat;
    (*this).eat()
    // println!("Eating...");
    // NOERROR
}

// Adjustor Thunks for IDomesticAnimal
// TODO: We need to consider abstracting this messy logic away from the production process.
unsafe extern "stdcall" fn idomesticanimal_query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = this.sub(1) as *mut BritishShortHairCat;
    (*this).query_interface(riid, ppv)
    // icat_query_interface(this.sub(1), riid, ppv)
}

unsafe extern "stdcall" fn idomesticanimal_add_ref(this: *mut IUnknownVPtr) -> u32 {
    let this = this.sub(1) as *mut BritishShortHairCat;
    (*this).add_ref()
    // icat_add_ref(this.sub(1))
}

unsafe extern "stdcall" fn idomesticanimal_release(this: *mut IUnknownVPtr) -> u32 {
    let this = this.sub(1) as *mut BritishShortHairCat;
    (*this).release()
}

unsafe extern "stdcall" fn idomesticanimal_eat(this: *mut IAnimalVPtr) -> HRESULT {
    let this = this.sub(1) as *mut BritishShortHairCat;
    (*this).eat()
    // println!("Eating...");
    // NOERROR
}

unsafe extern "stdcall" fn idomesticanimal_train(this: *mut IDomesticAnimalVPtr) -> HRESULT {
    let this = this.sub(1) as *mut BritishShortHairCat;
    (*this).train()
    // println!("Trainig...");
    // NOERROR
}

impl BritishShortHairCat {
    pub(crate) fn new() -> BritishShortHairCat {
        println!("Allocating new Vtable...");

        /* Initialising VTable for ICat */
        let icat_iunknown = IUnknownVTable {
            QueryInterface: icat_query_interface,
            Release: icat_release,
            AddRef: icat_add_ref,
        };
        let icat_ianimal = IAnimalVTable {
            base: icat_iunknown,
            Eat: icat_eat
        };

        let icat = ICatVTable {
            base:icat_ianimal,
            IgnoreHumans: icat_ignore_humans,
        };
        let icat_vptr = Box::into_raw(Box::new(icat));

        /* Initialising VTable for IDomesticAnimal */
        let idomesticanimal_iunknown = IUnknownVTable {
            QueryInterface: idomesticanimal_query_interface,
            Release: idomesticanimal_release,
            AddRef: idomesticanimal_add_ref,
        };
        let idomesticanimal_ianimal = IAnimalVTable {
            base: idomesticanimal_iunknown,
            Eat: idomesticanimal_eat
        };

        let idomesticanimal = IDomesticAnimalVTable {
            base: idomesticanimal_ianimal,
            Train: idomesticanimal_train,
        };

        let idomesticanimal_vptr = Box::into_raw(Box::new(idomesticanimal));

        BritishShortHairCat {
            inner_one: icat_vptr,
            inner_two: idomesticanimal_vptr,
            ref_count: 0,
        }
    }
}
