use winapi::shared::guiddef::IID;
use com::{ComInterface, ComPtr, IUnknown, IUnknownMethods};
use winapi::um::winnt::HRESULT;

pub const IID_IANIMAL: IID = IID {
    Data1: 0xeff8970e,
    Data2: 0xc50f,
    Data3: 0x45e0,
    Data4: [0x92, 0x84, 0x29, 0x1c, 0xe5, 0xa6, 0xf7, 0x71],
};

pub trait IAnimal: IUnknown {
    fn eat(&mut self) -> HRESULT;
}

unsafe impl ComInterface for IAnimal {
    const IID: IID = IID_IANIMAL;
}

pub type IAnimalVPtr = *const IAnimalVTable;

impl <T: IAnimal + ComInterface + ?Sized> IAnimal for ComPtr<T> {
    fn eat(&mut self) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut IAnimalVPtr;
        unsafe { ((**itf_ptr).1.Eat)(itf_ptr) }
    }
}

#[repr(C)]
pub struct IAnimalVTable(IUnknownMethods, IAnimalMethods);

#[allow(non_snake_case)]
#[repr(C)]
pub struct IAnimalMethods {
    pub Eat: unsafe extern "stdcall" fn(*mut IAnimalVPtr) -> HRESULT,
}
