use com::{ComInterface, ComPtr, IUnknownMethods, RawIUnknown};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_ILOCAL_FILE_MANAGER: IID = IID {
    Data1: 0x4fc333e3,
    Data2: 0xc389,
    Data3: 0x4c48,
    Data4: [0xb1, 0x08, 0x78, 0x95, 0xb0, 0xaf, 0x21, 0xad],
};

#[repr(C)]
pub struct ILocalFileManager {
    pub inner: RawILocalFileManager,
}

impl ILocalFileManager {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }

    pub fn delete_local(&mut self) {
        let _ = unsafe { self.inner.raw_delete_local() };
    }
}

unsafe impl ComInterface for ILocalFileManager {
    const IID: IID = IID_ILOCAL_FILE_MANAGER;
}

#[repr(C)]
pub struct RawILocalFileManager {
    pub vtable: *const ILocalFileManagerVTable,
}

impl RawILocalFileManager {
    unsafe fn raw_delete_local(&mut self) -> HRESULT {
        ((*self.vtable).1.DeleteLocal)(self as *mut RawILocalFileManager)
    }
}

impl std::convert::AsRef<RawIUnknown> for RawILocalFileManager {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawILocalFileManager as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawILocalFileManager {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawILocalFileManager as *mut RawIUnknown) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct ILocalFileManagerMethods {
    pub DeleteLocal: unsafe extern "stdcall" fn(*mut RawILocalFileManager) -> HRESULT,
}

#[repr(C)]
pub struct ILocalFileManagerVTable(pub IUnknownMethods, pub ILocalFileManagerMethods);
