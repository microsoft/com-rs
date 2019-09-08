use interface::ilocal_file_manager::ILocalFileManager;

use winapi::shared::winerror::{HRESULT, NOERROR};

use com::AggrCoClass;

/// The implementation class
#[repr(C)]
#[derive(AggrCoClass)]
#[com_implements(ILocalFileManager)]
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
        let init = InitLocalFileManager { user_field: 2 };
        LocalFileManager::allocate(init)
    }
}
