use com::{ComInterface, ComPtr, IClassFactoryMethods, IUnknownMethods, RawIUnknown};

use winapi::shared::guiddef::IID;

pub const IID_ICAT_CLASS: IID = IID {
    Data1: 0xf5353c58,
    Data2: 0xcfd9,
    Data3: 0x4204,
    Data4: [0x8d, 0x92, 0xd2, 0x74, 0xc7, 0x57, 0x8b, 0x53],
};

#[repr(C)]
pub struct ICatClass {
    pub inner: RawICatClass,
}

impl ICatClass {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }
}

unsafe impl ComInterface for ICatClass {
    const IID: IID = IID_ICAT_CLASS;
}

#[repr(C)]
pub struct RawICatClass {
    pub vtable: *const ICatClassVTable,
}

impl std::convert::AsRef<RawIUnknown> for RawICatClass {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawICatClass as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawICatClass {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawICatClass as *mut RawIUnknown) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct ICatClassMethods {}

#[repr(C)]
pub struct ICatClassVTable(
    pub IUnknownMethods,
    pub IClassFactoryMethods,
    pub ICatClassMethods,
);
