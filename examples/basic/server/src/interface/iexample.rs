use com::{ComInterface, ComPtr, IUnknownMethods, RawIUnknown, IID};

pub const IID_IEXAMPLE: IID = IID {
    data1: 0xC5F45CBC,
    data2: 0x4439,
    data3: 0x418C,
    data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};

#[repr(C)]
pub struct IExample {
    inner: RawIExample,
}

impl IExample {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }
}

unsafe impl ComInterface for IExample {
    const IID: IID = IID_IEXAMPLE;
}

#[repr(C)]
pub(crate) struct RawIExample {
    vtable: *const IExampleVTable,
}

impl RawIExample {}

impl std::convert::AsRef<RawIUnknown> for RawIExample {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawIExample as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawIExample {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawIExample as *mut RawIUnknown) }
    }
}

#[repr(C)]
pub struct IExampleMethods {}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IExampleVTable(pub IUnknownMethods, pub IExampleMethods);
