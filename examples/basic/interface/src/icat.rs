use super::ianimal::IAnimal;
use com::{ComInterface, ComPtr};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_ICAT: IID = IID {
    Data1: 0xf5353c58,
    Data2: 0xcfd9,
    Data3: 0x4204,
    Data4: [0x8d, 0x92, 0xd2, 0x74, 0xc7, 0x57, 0x8b, 0x53],
};

pub trait ICat: IAnimal {
    fn ignore_humans(&mut self) -> HRESULT;
}

unsafe impl ComInterface for ICat {
    type VTable = ICatVTable;
    const IID: IID = IID_ICAT;
}

pub type ICatVPtr = *const ICatVTable;

impl<T: ICat + ComInterface + ?Sized> ICat for ComPtr<T> {
    fn ignore_humans(&mut self) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ICatVPtr;
        unsafe { ((**itf_ptr).IgnoreHumans)(itf_ptr) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct ICatVTable {
    pub base: <IAnimal as ComInterface>::VTable,
    pub IgnoreHumans: unsafe extern "stdcall" fn(*mut ICatVPtr) -> HRESULT,
}

#[macro_export]
macro_rules! icat_gen_vtable {
    ($type:ty, $offset:literal) => {{
        let ianimal_vtable = ianimal_gen_vtable!($type, $offset);

        unsafe extern "stdcall" fn icat_ignore_humans(this: *mut ICatVPtr) -> HRESULT {
            let this = this.sub($offset) as *mut $type;
            (*this).ignore_humans()
        }

        ICatVTable {
            base: ianimal_vtable,
            IgnoreHumans: icat_ignore_humans,
        }
    }};
}
