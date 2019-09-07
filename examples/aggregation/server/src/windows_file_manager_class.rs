use crate::windows_file_manager::WindowsFileManager;
use com::{
    failed, IClassFactory, IClassFactoryVPtr, IClassFactoryVTable, IUnknown, IUnknownVPtr,
    IID_ICLASS_FACTORY, IID_IUNKNOWN,
};
use interface::CLSID_LOCAL_FILE_MANAGER_CLASS;

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFCLSID, REFIID},
        minwindef::{BOOL, LPVOID},
        winerror::{CLASS_E_NOAGGREGATION, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::combaseapi::CoCreateInstance,
};

#[repr(C)]
pub struct WindowsFileManagerClass {
    inner: IClassFactoryVPtr,
    ref_count: u32,
}

impl Drop for WindowsFileManagerClass {
    fn drop(&mut self) {
        println!("Dropping WindowsFileManagerClass");
        let _ = unsafe { Box::from_raw(self.inner as *mut IClassFactoryVTable) };
    }
}

impl IClassFactory for WindowsFileManagerClass {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        println!("Creating instance...");
        if aggr != std::ptr::null_mut() {
            return CLASS_E_NOAGGREGATION;
        }

        let mut wfm = WindowsFileManager::new();
        wfm.add_ref();
        let hr = wfm.query_interface(riid, ppv);
        wfm.release();

        Box::into_raw(wfm);
        hr
    }

    fn lock_server(&mut self, _increment: BOOL) -> HRESULT {
        println!("LockServer called");
        S_OK
    }
}

impl IUnknown for WindowsFileManagerClass {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        unsafe {
            println!("Querying interface on WindowsFileManagerClass...");

            let riid_ref = &*riid;
            if IsEqualGUID(riid_ref, &IID_IUNKNOWN) | IsEqualGUID(riid_ref, &IID_ICLASS_FACTORY) {
                *ppv = self as *const _ as *mut c_void;
                self.add_ref();
                NOERROR
            } else {
                E_NOINTERFACE
            }
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
            println!("Count is 0 for WindowsFileManagerClass. Freeing memory...");
            drop(self);
        }
        count
    }
}

impl WindowsFileManagerClass {
    pub(crate) fn new() -> WindowsFileManagerClass {
        println!("Allocating new Vtable for WindowsFileManagerClass...");
        let class_vtable = com::vtable!(WindowsFileManagerClass: IClassFactory);
        let vptr = Box::into_raw(Box::new(class_vtable));
        WindowsFileManagerClass {
            inner: vptr,
            ref_count: 0,
        }
    }
}
