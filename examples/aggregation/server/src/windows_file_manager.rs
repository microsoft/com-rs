use com::{failed, IUnknownVPtr, IID_IUNKNOWN};
use interface::{
    ifile_manager::IFileManager, ilocal_file_manager::ILocalFileManager,
    CLSID_LOCAL_FILE_MANAGER_CLASS,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{REFCLSID, REFIID},
        minwindef::LPVOID,
        winerror::{HRESULT, NOERROR},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::combaseapi::CoCreateInstance,
};

use com::co_class;
/// The implementation class
#[co_class(com_implements(IFileManager), aggr(ILocalFileManager))]
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

        // Instantiate object to aggregate
        // TODO: Create safe wrapper for instantiating as aggregate.
        let mut unknown_file_manager = std::ptr::null_mut::<c_void>();
        let hr = unsafe {
            CoCreateInstance(
                &CLSID_LOCAL_FILE_MANAGER_CLASS as REFCLSID,
                &*wfm as *const _ as winapi::um::unknwnbase::LPUNKNOWN,
                CLSCTX_INPROC_SERVER,
                &IID_IUNKNOWN as REFIID,
                &mut unknown_file_manager as *mut LPVOID,
            )
        };
        if failed(hr) {
            println!("Failed to instantiate aggregate! Error: {:x}", hr as u32);
            panic!();
        }

        wfm.set_aggregate_ilocal_file_manager(unknown_file_manager as *mut IUnknownVPtr);

        wfm
    }
}
