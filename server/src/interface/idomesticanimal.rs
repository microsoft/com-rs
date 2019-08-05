use super::ianimal::{IAnimalMethods, RawIAnimal};
use common::{ComInterface, ComPtr, IUnknownMethods, RawIUnknown, HRESULT, IID};

pub const IID_IDOMESTIC_ANIMAL: IID = IID {
    data1: 0xc22425df,
    data2: 0xefb2,
    data3: 0x4b85,
    data4: [0x93, 0x3e, 0x9c, 0xf7, 0xb2, 0x34, 0x59, 0xe8],
};

#[repr(C)]
pub struct IDomesticAnimal {
    pub(crate) inner: RawIDomesticAnimal,
}

impl IDomesticAnimal {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }

    pub fn eat(&mut self) {
        let inner: &mut RawIAnimal = self.inner.as_mut();
        inner.eat()
    }

    pub fn train(&mut self) {
        let _ = unsafe { self.inner.raw_train() };
    }
}

unsafe impl ComInterface for IDomesticAnimal {
    const IID: IID = IID_IDOMESTIC_ANIMAL;
}

#[repr(C)]
pub(crate) struct RawIDomesticAnimal {
    pub(crate) vtable: *const IDomesticAnimalVTable,
}

impl RawIDomesticAnimal {
    unsafe fn raw_train(&mut self) -> HRESULT {
        ((*self.vtable).2.Train)(self as *mut RawIDomesticAnimal)
    }
}

impl std::convert::AsRef<RawIUnknown> for RawIDomesticAnimal {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawIDomesticAnimal as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawIDomesticAnimal {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawIDomesticAnimal as *mut RawIUnknown) }
    }
}

impl std::convert::AsRef<RawIAnimal> for RawIDomesticAnimal {
    fn as_ref(&self) -> &RawIAnimal {
        unsafe { &*(self as *const RawIDomesticAnimal as *const RawIAnimal) }
    }
}

impl std::convert::AsMut<RawIAnimal> for RawIDomesticAnimal {
    fn as_mut(&mut self) -> &mut RawIAnimal {
        unsafe { &mut *(self as *mut RawIDomesticAnimal as *mut RawIAnimal) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IDomesticAnimalMethods {
    pub(crate) Train: unsafe extern "stdcall" fn(*mut RawIDomesticAnimal) -> HRESULT,
}
#[repr(C)]
pub struct IDomesticAnimalVTable(
    pub IUnknownMethods,
    pub IAnimalMethods,
    pub IDomesticAnimalMethods,
);
