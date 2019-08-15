use com::{ComInterface, ComPtr, IUnknownMethods, RawIUnknown};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_IFILE_MANAGER: IID = IID {
    Data1: 0x25a41124,
    Data2: 0x23d0,
    Data3: 0x46be,
    Data4: [0x83, 0x51, 0x04, 0x48, 0x89, 0xd5, 0xe3, 0x7e],
};

#[repr(C)]
pub struct IFileManager {
    pub inner: RawIFileManager,
}

impl IFileManager {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }

    pub fn delete_all(&mut self) {
        let _ = unsafe { self.inner.raw_delete_all() };
    }
}

unsafe impl ComInterface for IFileManager {
    const IID: IID = IID_IFILE_MANAGER;
}

#[repr(C)]
pub struct RawIFileManager {
    pub vtable: *const IFileManagerVTable,
}

impl RawIFileManager {
    unsafe fn raw_delete_all(&mut self) -> HRESULT {
        ((*self.vtable).1.DeleteAll)(self as *mut RawIFileManager)
    }
}

impl std::convert::AsRef<RawIUnknown> for RawIFileManager {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawIFileManager as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawIFileManager {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawIFileManager as *mut RawIUnknown) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IFileManagerMethods {
    pub DeleteAll: unsafe extern "stdcall" fn(*mut RawIFileManager) -> HRESULT,
}
#[repr(C)]
pub struct IFileManagerVTable(pub IUnknownMethods, pub IFileManagerMethods);
