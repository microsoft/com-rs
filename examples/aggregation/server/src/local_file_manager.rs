use interface::ilocal_file_manager::ILocalFileManager;

use winapi::shared::winerror::{HRESULT, NOERROR};

use com::co_class;

/// The implementation class
#[co_class(implements(ILocalFileManager), aggregatable)]
pub struct LocalFileManager {
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
        LocalFileManager::allocate(2)
    }
}
