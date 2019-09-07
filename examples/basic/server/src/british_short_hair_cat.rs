use interface::{
    ianimal::{IAnimal, IID_IANIMAL},
    icat::{ICat, ICatVPtr, ICatVTable, IID_ICAT},
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalVPtr, IDomesticAnimalVTable,
    },
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

use com::CoClass;

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[repr(C)]
#[derive(CoClass)]
#[com_implements(ICat, IDomesticAnimal)]
pub struct InitBritishShortHairCat {
    num_owners: u32,
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

impl BritishShortHairCat {
    pub(crate) fn new() -> Box<BritishShortHairCat> {
        let init = InitBritishShortHairCat {
            num_owners: 20
        };
        BritishShortHairCat::allocate(init)
    }
}

// -----------------------  GENERATED  ----------------------------
// #[repr(C)]
// pub struct BritishShortHairCat {
//     // inner must always be first because Cat is actually an ICat with one extra field at the end
//     icat: ICatVPtr,
//     idomesticanimal: IDomesticAnimalVPtr,
//     ref_count: u32,
//     value: InitBritishShortHairCat
// }

// impl Deref for BritishShortHairCat {
//     type Target = InitBritishShortHairCat;
//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }

// impl Drop for BritishShortHairCat {
//     fn drop(&mut self) {
//         let _ = unsafe {
//             Box::from_raw(self.icat as *mut ICatVTable);
//             Box::from_raw(self.idomesticanimal as *mut IDomesticAnimalVTable);
//         };
//     }
// }

// impl IUnknown for BritishShortHairCat {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
//         unsafe {
//             let riid = &*riid;

//             if IsEqualGUID(riid, &IID_IUNKNOWN)
//                 | IsEqualGUID(riid, &IID_ICAT)
//                 | IsEqualGUID(riid, &IID_IANIMAL)
//             {
//                 *ppv = &self.icat as *const _ as *mut c_void;
//             } else if IsEqualGUID(riid, &IID_IDOMESTIC_ANIMAL) {
//                 *ppv = &self.idomesticanimal as *const _ as *mut c_void;
//             } else {
//                 println!("Returning NO INTERFACE.");
//                 return E_NOINTERFACE;
//             }

//             println!("Successful!.");
//             self.add_ref();
//             NOERROR
//         }
//     }

//     fn add_ref(&mut self) -> u32 {
//         self.ref_count += 1;
//         println!("Count now {}", self.ref_count);
//         self.ref_count
//     }

//     fn release(&mut self) -> u32 {
//         self.ref_count -= 1;
//         println!("Count now {}", self.ref_count);
//         let count = self.ref_count;
//         if count == 0 {
//             println!("Count is 0 for BritishShortHairCat. Freeing memory...");
//             drop(self)
//         }
//         count
//     }
// }

// impl BritishShortHairCat {
//     fn allocate(value: InitBritishShortHairCat) -> Box<BritishShortHairCat> {
//         println!("Allocating new vtable for Cat...");
//         let icat_vtable = com::vtable!(BritishShortHairCat: ICat);
//         let icat_vptr = Box::into_raw(Box::new(icat_vtable));
//         let idomesticanimal_vtable = com::vtable!(BritishShortHairCat: IDomesticAnimal, 1);
//         let idomesticanimal_vptr = Box::into_raw(Box::new(idomesticanimal_vtable));

//         let out = BritishShortHairCat {
//             icat: icat_vptr,
//             idomesticanimal: idomesticanimal_vptr,
//             ref_count: 0,
//             value
//         };
//         Box::new(out)
//     }
// }
