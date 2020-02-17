use com::com_interface;
use winapi::um::winnt::HRESULT;

use crate::IAnimal;

#[com_interface("C22425DF-EFB2-4B85-933E-9CF7B23459E8")]
pub trait IDomesticAnimal: IAnimal {
    unsafe fn train(&self) -> HRESULT;
}
