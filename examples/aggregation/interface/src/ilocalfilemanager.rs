use com::{ComInterface, ComPtr, IUnknown};

use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub const IID_ILOCAL_FILE_MANAGER: IID = IID {
    Data1: 0x4fc333e3,
    Data2: 0xc389,
    Data3: 0x4c48,
    Data4: [0xb1, 0x08, 0x78, 0x95, 0xb0, 0xaf, 0x21, 0xad],
};

pub trait ILocalFileManager: IUnknown {
    fn delete_local(&mut self) -> HRESULT;
}

unsafe impl ComInterface for dyn ILocalFileManager {
    type VTable = ILocalFileManagerVTable;
    const IID: IID = IID_ILOCAL_FILE_MANAGER;
}

pub type ILocalFileManagerVPtr = *const ILocalFileManagerVTable;

impl<T: ILocalFileManager + ComInterface + ?Sized> ILocalFileManager for ComPtr<T> {
    fn delete_local(&mut self) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ILocalFileManagerVPtr;
        unsafe { ((**itf_ptr).DeleteLocal)(itf_ptr) }
    }
}
#[allow(non_snake_case)]
#[repr(C)]
pub struct ILocalFileManagerVTable {
    pub base: <dyn IUnknown as ComInterface>::VTable,
    pub DeleteLocal: unsafe extern "stdcall" fn(*mut ILocalFileManagerVPtr) -> HRESULT,
}
