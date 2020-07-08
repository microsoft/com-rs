use com::{com_interface, interfaces::iunknown::IUnknown, sys::HRESULT};

com_interface! {
    #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
    pub unsafe interface IAnimal: IUnknown {
        unsafe fn eat(&self) -> HRESULT;
    }
}
