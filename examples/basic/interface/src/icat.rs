use com::com_interface;
use winapi::um::winnt::HRESULT;

use crate::IAnimal;

#[com_interface(F5353C58-CFD9-4204-8D92-D274C7578B53)]
pub trait ICat: IAnimal {
    fn ignore_humans(&self) -> HRESULT;
}
