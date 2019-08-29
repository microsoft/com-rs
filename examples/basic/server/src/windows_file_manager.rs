use com::{failed, IUnknownVPtr, IID_IUNKNOWN, ComPtr, IUnknown, IUnknownVTable};
use interface::{
    ifilemanager::{
        IFileManager, IFileManagerVTable, IFileManagerVPtr, IID_IFILE_MANAGER,
    },
    ilocalfilemanager::IID_ILOCAL_FILE_MANAGER,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

use std::ptr::NonNull;
use std::mem::forget;

/// The implementation class
#[repr(C)]
pub struct WindowsFileManager {
    inner_one: IFileManagerVPtr,
    ref_count: u32,
    pub lfm_iunknown: *mut IUnknownVPtr,
}

impl Drop for WindowsFileManager {
    fn drop(&mut self) {
        unsafe {
            let mut lfm_iunknown : ComPtr<IUnknown> = ComPtr::new(NonNull::new(self.lfm_iunknown as *mut c_void).unwrap());
            lfm_iunknown.release();
            Box::from_raw(self.inner_one as *mut IFileManagerVTable);

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

impl IUnknown for WindowsFileManager {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;
            if IsEqualGUID(riid, &IID_IUNKNOWN) | IsEqualGUID(riid, &IID_IFILE_MANAGER) {
                *ppv = self as *const _ as *mut c_void;
            } else if IsEqualGUID(riid, &IID_ILOCAL_FILE_MANAGER) {

                let mut lfm_iunknown : ComPtr<IUnknown> = ComPtr::new(NonNull::new(self.lfm_iunknown as *mut c_void).unwrap());
                let hr = lfm_iunknown.query_interface(riid, ppv);
                if failed(hr) {
                    return E_NOINTERFACE;
                }

                // We release it as the previous call add_ref-ed the inner object.
                // The intention is to transfer reference counting logic to the
                // outer object.
                lfm_iunknown.release();

                forget(lfm_iunknown);
            } else {
                return E_NOINTERFACE;
            }

            self.add_ref();
            NOERROR
        }
    }

    fn add_ref(&mut self) -> u32 {
        self.ref_count += 1;
        println!("Count now {}", self.ref_count);
        self.ref_count
    }

    fn release(&mut self) -> u32 {
        self.ref_count -= 1;
        println!("Count now {}", self.ref_count);
        let count = self.ref_count;
        if count == 0 {
            println!("Count is 0 for WindowsFileManager. Freeing memory...");
            drop(self);
        }
        count
    }
}

unsafe extern "stdcall" fn ifilemanager_query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = this as *mut WindowsFileManager;
    (*this).query_interface(riid, ppv)
}

unsafe extern "stdcall" fn ifilemanager_add_ref(this: *mut IUnknownVPtr) -> u32 {
    let this = this as *mut WindowsFileManager;
    (*this).add_ref()
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn ifilemanager_release(this: *mut IUnknownVPtr) -> u32 {
    let this = this as *mut WindowsFileManager;
    (*this).release()
}

unsafe extern "stdcall" fn delete_all(this: *mut IFileManagerVPtr) -> HRESULT {
    let this = this as *mut WindowsFileManager;
    (*this).delete_all()
}

impl WindowsFileManager {
    pub(crate) fn new() -> WindowsFileManager {
        println!("Allocating new Vtable...");

        // Initialising VTable for IFileManager
        let ifilemanager_iunknown = IUnknownVTable {
            QueryInterface: ifilemanager_query_interface,
            Release: ifilemanager_release,
            AddRef: ifilemanager_add_ref,
        };

        let ifilemanager = IFileManagerVTable {
            base: ifilemanager_iunknown,
            DeleteAll: delete_all,
        };
        let ifilemanager_vptr = Box::into_raw(Box::new(ifilemanager));

        let out = WindowsFileManager {
            inner_one: ifilemanager_vptr,
            ref_count: 0,
            lfm_iunknown: std::ptr::null_mut::<IUnknownVPtr>(),
        };

        out
    }
}
