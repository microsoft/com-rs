use super::ianimal::{IAnimalMethods, RawIAnimal};
use com::{ComInterface, ComPtr, IUnknownMethods, RawIUnknown};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_ICAT: IID = IID {
    Data1: 0xf5353c58,
    Data2: 0xcfd9,
    Data3: 0x4204,
    Data4: [0x8d, 0x92, 0xd2, 0x74, 0xc7, 0x57, 0x8b, 0x53],
};

#[repr(C)]
pub struct ICat {
    pub inner: RawICat,
}

impl ICat {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }

    pub fn eat(&mut self) {
        let inner: &mut RawIAnimal = self.inner.as_mut();
        inner.eat()
    }

    pub fn ignore_humans(&mut self) {
        let _ = unsafe { self.inner.raw_ignore_humans() };
    }
}

unsafe impl ComInterface for ICat {
    const IID: IID = IID_ICAT;
}

#[repr(C)]
pub struct RawICat {
    pub vtable: *const ICatVTable,
}

impl RawICat {
    pub unsafe fn raw_ignore_humans(&mut self) -> HRESULT {
        ((*self.vtable).2.IgnoreHumans)(self as *mut RawICat)
    }
}

impl std::convert::AsRef<RawIUnknown> for RawICat {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawICat as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawICat {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawICat as *mut RawIUnknown) }
    }
}

impl std::convert::AsRef<RawIAnimal> for RawICat {
    fn as_ref(&self) -> &RawIAnimal {
        unsafe { &*(self as *const RawICat as *const RawIAnimal) }
    }
}

impl std::convert::AsMut<RawIAnimal> for RawICat {
    fn as_mut(&mut self) -> &mut RawIAnimal {
        unsafe { &mut *(self as *mut RawICat as *mut RawIAnimal) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct ICatMethods {
    pub IgnoreHumans: unsafe extern "stdcall" fn(*mut RawICat) -> HRESULT,
}

#[repr(C)]
pub struct ICatVTable(pub IUnknownMethods, pub IAnimalMethods, pub ICatMethods);
