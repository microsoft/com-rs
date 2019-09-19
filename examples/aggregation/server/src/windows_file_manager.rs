use com::{interfaces::iunknown::IUnknown, ApartmentThreadedRuntime as Runtime};

use interface::{
    ifile_manager::IFileManager, ilocal_file_manager::ILocalFileManager,
    CLSID_LOCAL_FILE_MANAGER_CLASS,
};

use winapi::shared::winerror::{HRESULT, NOERROR};

use com::co_class;

/// The implementation class
#[co_class(implements(IFileManager), aggregates(ILocalFileManager))]
pub struct WindowsFileManager {
    user_field: u32,
}

impl IFileManager for WindowsFileManager {
    fn delete_all(&self) -> HRESULT {
        println!("Deleting all by delegating to Local and Remote File Managers...");
        NOERROR
    }
}

impl WindowsFileManager {
    pub(crate) fn new() -> Box<WindowsFileManager> {
        let mut wfm = WindowsFileManager::allocate(20);
        let runtime = Runtime::new().expect("Failed to get runtime!");
        let iunknown = runtime
            .create_aggregated_instance::<dyn IUnknown, WindowsFileManager>(
                &CLSID_LOCAL_FILE_MANAGER_CLASS,
                &mut *wfm,
            )
            .expect("Failed to instantiate aggregate!");

        wfm.set_aggregate_ilocal_file_manager(iunknown);

        wfm
    }
}
