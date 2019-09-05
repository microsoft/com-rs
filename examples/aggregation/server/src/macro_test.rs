use com_interface_attribute::AggrCoClass;

use com::{ComPtr, IUnknown, IUnknownVPtr, IUnknownVTable, IID_IUNKNOWN, iunknown_gen_vtable,};
use interface::{
    ilocal_file_manager::{
        ILocalFileManager, ILocalFileManagerVPtr, ILocalFileManagerVTable, IID_ILOCAL_FILE_MANAGER,
    },
    ilocal_file_manager_gen_vtable,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID,},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

use core::mem::forget;

#[derive(AggrCoClass)]
#[com_implements(ILocalFileManager)]
#[repr(C)]
pub struct InitLocalFileManager {
    user_field: u32,
}

impl ILocalFileManager for LocalFileManager {
    fn delete_local(&mut self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl LocalFileManager {
    pub(crate) fn new() -> Box<LocalFileManager> {
        let init = InitLocalFileManager {
            user_field: 2,
        };
        LocalFileManager::allocate(init)
    }
}