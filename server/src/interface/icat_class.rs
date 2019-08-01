use common::{ComInterface, ComPtr, IClassFactoryVTable, RawIUnknown, IID};

pub const IID_ICAT_CLASS: IID = IID {
    data1: 0xf5353c58,
    data2: 0xcfd9,
    data3: 0x4204,
    data4: [0x8d, 0x92, 0xd2, 0x74, 0xc7, 0x57, 0x8b, 0x53],
};

#[repr(C)]
pub struct ICatClass {
    pub(crate) inner: RawICatClass,
}

impl ICatClass {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }
}

impl ComInterface for ICatClass {
    const IID: IID = IID_ICAT_CLASS;
}

#[repr(C)]
pub(crate) struct RawICatClass {
    pub(crate) vtable: *const ICatClassVTable,
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
pub struct ICatClassVTable {
    pub(crate) iclassfactory: IClassFactoryVTable,
}
