use com::interfaces;
use com::sys::HRESULT;

use crate::IAnimal;

interfaces! {
    #[uuid("C22425DF-EFB2-4B85-933E-9CF7B23459E8")]
    pub unsafe interface IDomesticAnimal: IAnimal {
        pub fn train(&self) -> HRESULT;
    }
}
