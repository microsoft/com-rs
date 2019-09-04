use com_interface_attribute::CoClass;

use com::iunknown_gen_vtable;

use interface::{
    ianimal::{IAnimal, IAnimalVPtr, IAnimalVTable, IID_IANIMAL},
    ianimal_gen_vtable,
    icat::{ICat, ICatVPtr, ICatVTable, IID_ICAT},
    icat_gen_vtable, idomestic_animal_gen_vtable,
    idomesticanimal::{
        IDomesticAnimal, IDomesticAnimalVPtr, IDomesticAnimalVTable, IID_IDOMESTIC_ANIMAL,
    },
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

#[derive(CoClass)]
#[com_implements(ICat, IDomesticAnimal)]
pub struct InitBritishShortHairCat {
    num_owners: u32,
}