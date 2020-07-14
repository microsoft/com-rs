use com::com_interface;
use com::sys::HRESULT;

use crate::IAnimal;

com_interface! {
    #[uuid("C22425DF-EFB2-4B85-933E-9CF7B23459E8")]
    pub unsafe interface IDomesticAnimal: IAnimal {
        fn train(&self) -> HRESULT;
    }
}
