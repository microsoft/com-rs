use interface::ilocal_file_manager::ILocalFileManager;

use com::co_class;
use com::sys::{HRESULT, NOERROR};

/// The implementation class
#[co_class(implements(ILocalFileManager), aggregatable)]
pub struct LocalFileManager {
    user_field: u32,
}

impl ILocalFileManager for LocalFileManager {
    unsafe fn delete_local(&self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl LocalFileManager {
    pub(crate) fn new() -> Box<LocalFileManager> {
        LocalFileManager::allocate(2)
    }
}
