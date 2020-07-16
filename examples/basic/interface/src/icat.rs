use com::com_interface;
use com::sys::HRESULT;

use crate::IAnimal;

com_interface! {
    #[uuid("F5353C58-CFD9-4204-8D92-D274C7578B53")]
    pub unsafe interface ICat: IAnimal {
        pub fn ignore_humans(&self) -> HRESULT;
    }
}
