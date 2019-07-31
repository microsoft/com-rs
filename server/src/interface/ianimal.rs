use super::icat::{ICatVTable, RawICat};
use common::{ComInterface, ComPtr, RawIUnknown, HRESULT, IID};

pub const IID_IANIMAL: IID = IID {
    data1: 0xeff8970e,
    data2: 0xc50f,
    data3: 0x45e0,
    data4: [0x92, 0x84, 0x29, 0x1c, 0xe5, 0xa6, 0xf7, 0x71],
};

#[repr(C)]
pub struct IAnimal {
    inner: RawIAnimal,
}

impl IAnimal {
    pub fn eat(&mut self) {
        self.inner.eat()
    }

    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }
}

impl ComInterface for IAnimal {
    const IID: IID = IID_IANIMAL;
}

#[repr(C)]
pub(crate) struct RawIAnimal {
    vtable: *const ICatVTable,
}

impl RawIAnimal {
    pub fn eat(&mut self) {
        let _ = unsafe { self.raw_eat() };
    }

    pub unsafe fn raw_eat(&mut self) -> HRESULT {
        ((*self.vtable).Eat)(self as *mut RawIAnimal as *mut RawICat)
    }
}

impl std::convert::AsRef<RawIUnknown> for RawIAnimal {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawIAnimal as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawIAnimal {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawIAnimal as *mut RawIUnknown) }
    }
}
