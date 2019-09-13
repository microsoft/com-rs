use com::{com_interface, IUnknown};
use winapi::um::winnt::HRESULT;

#[com_interface(EFF8970E-C50F-45E0-9284-291CE5A6F771)]
pub trait IAnimal: IUnknown {
    fn eat(&self) -> HRESULT;
}
