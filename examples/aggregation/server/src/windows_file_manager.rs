use com::{
    interfaces::iunknown::IUnknown,
    runtime::{create_aggregated_instance, init_runtime},
};

use interface::{
    ifile_manager::IFileManager, ilocal_file_manager::ILocalFileManager,
    CLSID_LOCAL_FILE_MANAGER_CLASS,
};

use com::co_class;
use com::sys::{HRESULT, NOERROR};

/// The implementation class
#[co_class(implements(IFileManager), aggregates(ILocalFileManager))]
pub struct WindowsFileManager {
    user_field: u32,
}

impl IFileManager for WindowsFileManager {
    unsafe fn delete_all(&self) -> HRESULT {
        println!("Deleting all by delegating to Local and Remote File Managers...");
        NOERROR
    }
}

impl WindowsFileManager {
    pub(crate) fn new() -> Box<WindowsFileManager> {
        let mut wfm = WindowsFileManager::allocate(20);
        init_runtime().expect("Failed to get runtime!");
        let iunknown = create_aggregated_instance::<dyn IUnknown, WindowsFileManager>(
            &CLSID_LOCAL_FILE_MANAGER_CLASS,
            &mut *wfm,
        )
        .expect("Failed to instantiate aggregate!");

        wfm.set_aggregate_ilocal_file_manager(iunknown);

        wfm
    }
}
