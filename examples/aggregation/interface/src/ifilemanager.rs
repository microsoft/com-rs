use com::{ComInterface, ComPtr, IUnknown};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_IFILE_MANAGER: IID = IID {
    Data1: 0x25a41124,
    Data2: 0x23d0,
    Data3: 0x46be,
    Data4: [0x83, 0x51, 0x04, 0x48, 0x89, 0xd5, 0xe3, 0x7e],
};

pub trait IFileManager: IUnknown {
    fn delete_all(&mut self) -> HRESULT;
}

unsafe impl ComInterface for IFileManager {
    type VTable = IFileManagerVTable;
    const IID: IID = IID_IFILE_MANAGER;
}

pub type IFileManagerVPtr = *const IFileManagerVTable;

impl<T: IFileManager + ComInterface + ?Sized> IFileManager for ComPtr<T> {
    fn delete_all(&mut self) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut IFileManagerVPtr;
        unsafe { ((**itf_ptr).DeleteAll)(itf_ptr) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IFileManagerVTable {
    pub base: <IUnknown as ComInterface>::VTable,
    pub DeleteAll: unsafe extern "stdcall" fn(*mut IFileManagerVPtr) -> HRESULT,
}
