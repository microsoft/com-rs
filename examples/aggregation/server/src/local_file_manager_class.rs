use crate::local_file_manager::LocalFileManager;

use com::{
    ComPtr, IClassFactory, IClassFactoryVPtr, IClassFactoryVTable, IUnknown, IUnknownVPtr,
    IID_ICLASS_FACTORY, IID_IUNKNOWN,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID},
        minwindef::BOOL,
        winerror::{E_INVALIDARG, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
    },
};

use core::mem::forget;

#[repr(C)]
pub struct LocalFileManagerClass {
    inner: IClassFactoryVPtr,
    ref_count: u32,
}

impl Drop for LocalFileManagerClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner as *mut IClassFactoryVTable) };
    }
}

impl IClassFactory for LocalFileManagerClass {
    fn create_instance(
        &mut self,
        aggr: *mut IUnknownVPtr,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        println!("Creating instance...");

        unsafe {
            let riid = &*riid;

            if !aggr.is_null() && !IsEqualGUID(riid, &IID_IUNKNOWN) {
                *ppv = std::ptr::null_mut::<c_void>();
                return E_INVALIDARG;
            }

            let mut lfm = Box::new(LocalFileManager::new());
            // This check has to be here because it can only be done after object
            // is allocated on the heap (address of nonDelegatingUnknown fixed)
            lfm.set_iunknown(aggr);

            // As an aggregable object, we have to add_ref through the
            // non-delegating IUnknown on creation. Otherwise, we might
            // add_ref the outer object if aggregated.
            lfm.inner_add_ref();
            let hr = lfm.inner_query_interface(riid, ppv);
            lfm.inner_release();

            Box::into_raw(lfm);
            hr
        }
    }

    fn lock_server(&mut self, _increment: BOOL) -> HRESULT {
        println!("LockServer called");
        S_OK
    }
}

impl IUnknown for LocalFileManagerClass {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            println!("Querying interface on LocalFileManagerClass...");

            let riid = &*riid;
            if IsEqualGUID(riid, &IID_IUNKNOWN) | IsEqualGUID(riid, &IID_ICLASS_FACTORY) {
                *ppv = self as *const _ as *mut c_void;
                self.add_ref();
                NOERROR
            } else {
                E_NOINTERFACE
            }
        }
    }

    fn add_ref(&mut self) -> u32 {
        println!("Adding ref...");
        self.ref_count += 1;
        println!("Count now {}", self.ref_count);
        self.ref_count
    }

    fn release(&mut self) -> u32 {
        println!("Releasing...");
        self.ref_count -= 1;
        println!("Count now {}", self.ref_count);
        let count = self.ref_count;
        if count == 0 {
            println!("Count is 0 for LocalFileManagerClass. Freeing memory...");
            drop(self);
        }
        count
    }
}

impl LocalFileManagerClass {
    pub(crate) fn new() -> LocalFileManagerClass {
        let iclass_factory = com::vtable!(LocalFileManagerClass: IClassFactory);
        let vptr = Box::into_raw(Box::new(iclass_factory));
        LocalFileManagerClass {
            inner: vptr,
            ref_count: 0,
        }
    }
}
