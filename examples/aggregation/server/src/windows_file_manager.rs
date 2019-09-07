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
use std::ops::{DerefMut,};

use com::CoClass;

/// The implementation class
#[derive(CoClass)]
#[com_implements(IFileManager)]
#[repr(C)]
pub struct InitWindowsFileManager {
    #[aggr(ILocalFileManager)]
    lfm_iunknown: *mut IUnknownVPtr,
}

impl DerefMut for WindowsFileManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.__init_struct
    }
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

// ------------------------- MACRO GENERATED --------------------------------

// #[repr(C)]
// pub struct WindowsFileManager {
//     ifilemanager: IFileManagerVPtr,
//     ref_count: u32,
//     value: InitWindowsFileManager,
// }

// impl Drop for WindowsFileManager {
//     fn drop(&mut self) {
//         unsafe {
//             Box::from_raw(self.ifilemanager as *mut IFileManagerVTable);
//         };
//     }
// }

// impl IUnknown for WindowsFileManager {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
//         unsafe {
//             let riid = &*riid;
//             if IsEqualGUID(riid, &IID_IUNKNOWN) | IsEqualGUID(riid, &IID_IFILE_MANAGER) {
//                 *ppv = self as *const _ as *mut c_void;
//             } else if IsEqualGUID(riid, &IID_ILOCAL_FILE_MANAGER) {
//                 let mut lfm_iunknown: ComPtr<dyn IUnknown> =
//                     ComPtr::new(self.lfm_iunknown as *mut c_void);
//                 let hr = lfm_iunknown.query_interface(riid, ppv);
//                 if failed(hr) {
//                     return E_NOINTERFACE;
//                 }

//                 // We release it as the previous call add_ref-ed the inner object.
//                 // The intention is to transfer reference counting logic to the
//                 // outer object.
//                 lfm_iunknown.release();

//                 forget(lfm_iunknown);
//             } else {
//                 return E_NOINTERFACE;
//             }

//             self.add_ref();
//             NOERROR
//         }
//     }

//     fn add_ref(&mut self) -> u32 {
//         self.ref_count += 1;
//         println!("Count now {}", self.ref_count);
//         self.ref_count
//     }

//     fn release(&mut self) -> u32 {
//         self.ref_count -= 1;
//         println!("Count now {}", self.ref_count);
//         let count = self.ref_count;
//         if count == 0 {
//             println!("Count is 0 for WindowsFileManager. Freeing memory...");
//             unsafe { Box::from_raw(self as *const _ as *mut WindowsFileManager); }
//         }
//         count
//     }
// }

// impl WindowsFileManager {
//     fn allocate(value: InitWindowsFileManager) -> Box<WindowsFileManager> {
//         println!("Allocating new Vtable...");

//         // Initialising VTable for IFileManager
//         let ifilemanager = ifile_manager_gen_vtable!(WindowsFileManager, 0);
//         let ifilemanager_vptr = Box::into_raw(Box::new(ifilemanager));

//         let wfm = WindowsFileManager {
//             ifilemanager: ifilemanager_vptr,
//             ref_count: 0,
//             value
//         };

//         Box::new(wfm)
//     }
// }
// 
// impl Deref for WindowsFileManager {
//     type Target = InitWindowsFileManager;
//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
