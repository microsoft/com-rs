use com_interface_attribute::CoClass;

use com::iunknown_gen_vtable;

use interface::{
    ianimal::{IAnimal, IAnimalVPtr, IAnimalVTable,},
    ianimal_gen_vtable,
    icat::{ICat, ICatVPtr, ICatVTable,},
    icat_gen_vtable, idomestic_animal_gen_vtable,
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalVPtr, IDomesticAnimalVTable,
    },
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, REFIID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

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