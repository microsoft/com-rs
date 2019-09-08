use com::{failed, ComPtr, IUnknown, IUnknownVPtr, IID_IUNKNOWN};
use interface::{
    ifile_manager::{IFileManager},
    ilocal_file_manager::{ILocalFileManager},
    CLSID_LOCAL_FILE_MANAGER_CLASS,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{REFCLSID, REFIID,},
        winerror::{HRESULT, NOERROR},
        minwindef::{LPVOID, },
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::CoCreateInstance
    },
};

use std::mem::forget;

use com::CoClass;

/// The implementation class
#[derive(CoClass)]
#[com_implements(IFileManager)]
#[repr(C)]
pub struct InitWindowsFileManager {
    #[aggr(ILocalFileManager)]
    lfm_iunknown: *mut IUnknownVPtr,
}

impl Drop for InitWindowsFileManager {
    fn drop(&mut self) {
        unsafe {
            println!("Dropping init struct..");
            let mut lfm_iunknown: ComPtr<dyn IUnknown> =
                ComPtr::new(self.lfm_iunknown as *mut c_void);
            lfm_iunknown.release();
            forget(lfm_iunknown);
        };
    }
    
}

impl IFileManager for WindowsFileManager {
    fn delete_all(&mut self) -> HRESULT {
        println!("Deleting all by delegating to Local and Remote File Managers...");
        NOERROR
    }
}

impl WindowsFileManager {
    pub(crate) fn new() -> Box<WindowsFileManager> {
        let init = InitWindowsFileManager {
            lfm_iunknown: std::ptr::null_mut::<IUnknownVPtr>()
        };

        let mut wfm = WindowsFileManager::allocate(init);

        // Instantiate object to aggregate
        // TODO: Should change to use safe ComPtr methods instead.
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

        wfm.lfm_iunknown = unknown_file_manager as *mut IUnknownVPtr;

        wfm
    }
}