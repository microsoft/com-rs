use std::os::raw::c_void;

use com::{
    failed, IUnknownMethods, RawIUnknown, E_NOINTERFACE, HRESULT, IID, IID_IUNKNOWN, NOERROR,
};
use interface::{
    ifilemanager::{
        IFileManager, IFileManagerMethods, IFileManagerVTable, RawIFileManager, IID_IFILE_MANAGER,
    },
    ilocalfilemanager::IID_ILOCAL_FILE_MANAGER,
};

/// The implementation class
#[repr(C)]
pub struct WindowsFileManager {
    inner_one: IFileManager,
    ref_count: u32,
    pub p_unk_local_file_manager: *mut RawIUnknown,
}

impl Drop for WindowsFileManager {
    fn drop(&mut self) {
        unsafe {
            (*self.p_unk_local_file_manager).raw_release();
            Box::from_raw(self.inner_one.inner.vtable as *mut IFileManagerVTable)
        };
    }
}

unsafe extern "stdcall" fn ifilemanager_query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let obj = this as *mut WindowsFileManager;

    match *riid {
        IID_IUNKNOWN | IID_IFILE_MANAGER => {
            *ppv = this as *mut c_void;
        }
        IID_ILOCAL_FILE_MANAGER => {
            let hr = (*((*obj).p_unk_local_file_manager)).raw_query_interface(riid, ppv);
            if failed(hr) {
                return E_NOINTERFACE;
            }

            // We release it as the previous call add_ref-ed the inner object.
            // The intention is to transfer reference counting logic to the
            // outer object.
            (*((*obj).p_unk_local_file_manager)).raw_release();
        }
        _ => {
            return E_NOINTERFACE;
        }
    }

    (*this).raw_add_ref();
    NOERROR
}

unsafe extern "stdcall" fn ifilemanager_add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this as *mut WindowsFileManager;
    (*this).ref_count += 1;
    println!("WFM Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn ifilemanager_release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this as *mut WindowsFileManager;
    (*this).ref_count -= 1;
    println!("WFM Count now {}", (*this).ref_count);
    let count = (*this).ref_count;
    if count == 0 {
        println!("Count is 0. Freeing memory...");
        let _ = Box::from_raw(this);
    }
    count
}

unsafe extern "stdcall" fn delete_all(_this: *mut RawIFileManager) -> HRESULT {
    println!("Deleting all by delegating to Local and Remote File Managers...");
    NOERROR
}

impl WindowsFileManager {
    pub(crate) fn new() -> WindowsFileManager {
        println!("Allocating new Vtable...");

        // Initialising VTable for IFileManager
        let ifilemanager_iunknown = IUnknownMethods {
            QueryInterface: ifilemanager_query_interface,
            Release: ifilemanager_release,
            AddRef: ifilemanager_add_ref,
        };

        let ifilemanager = IFileManagerMethods {
            DeleteAll: delete_all,
        };
        let ifilemanager_vtable = Box::into_raw(Box::new(IFileManagerVTable(
            ifilemanager_iunknown,
            ifilemanager,
        )));
        let ifilemanager_inner = RawIFileManager {
            vtable: ifilemanager_vtable,
        };

        let out = WindowsFileManager {
            inner_one: IFileManager {
                inner: ifilemanager_inner,
            },
            ref_count: 0,
            p_unk_local_file_manager: std::ptr::null_mut::<RawIUnknown>(),
        };

        out
    }
}
