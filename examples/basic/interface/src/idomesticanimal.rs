use super::ianimal::IAnimal;
use com::{ComInterface, ComPtr};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_IDOMESTIC_ANIMAL: IID = IID {
    Data1: 0xc22425df,
    Data2: 0xefb2,
    Data3: 0x4b85,
    Data4: [0x93, 0x3e, 0x9c, 0xf7, 0xb2, 0x34, 0x59, 0xe8],
};

pub trait IDomesticAnimal: IAnimal {
    fn train(&mut self) -> HRESULT;
}

unsafe impl ComInterface for dyn IDomesticAnimal {
    type VTable = IDomesticAnimalVTable;
    const IID: IID = IID_IDOMESTIC_ANIMAL;
}

pub type IDomesticAnimalVPtr = *const IDomesticAnimalVTable;

impl<T: IDomesticAnimal + ComInterface + ?Sized> IDomesticAnimal for ComPtr<T> {
    fn train(&mut self) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut IDomesticAnimalVPtr;
        unsafe { ((**itf_ptr).Train)(itf_ptr) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IDomesticAnimalVTable {
    pub base: <dyn IAnimal as ComInterface>::VTable,
    pub Train: unsafe extern "stdcall" fn(*mut IDomesticAnimalVPtr) -> HRESULT,
}

#[macro_export]
macro_rules! idomesticanimal_gen_vtable {
    ($type:ty, $offset:literal) => {{
        let ianimal_vtable = ianimal_gen_vtable!($type, $offset);

        unsafe extern "stdcall" fn idomesticanimal_train(
            this: *mut IDomesticAnimalVPtr,
        ) -> HRESULT {
            let this = this.sub($offset) as *mut $type;
            (*this).train()
        }

        IDomesticAnimalVTable {
            base: ianimal_vtable,
            Train: idomesticanimal_train,
        }
    }};
}
