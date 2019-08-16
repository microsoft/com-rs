use com::{IUnknownMethods, IUnknownVTable, IUnknown, IUnknownVPtr, IID_IUNKNOWN, ComPtr,};
use interface::ilocalfilemanager::{
    ILocalFileManager, ILocalFileManagerMethods, ILocalFileManagerVTable, ILocalFileManagerVPtr,
    IID_ILOCAL_FILE_MANAGER,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

use std::ptr::NonNull;
use core::mem::forget;

/// The implementation class
#[repr(C)]
pub struct LocalFileManager {
    inner_one: ILocalFileManagerVPtr,
    pub non_delegating_unk: IUnknownVPtr,
    pub iunk_to_use: *mut IUnknownVPtr,
    ref_count: u32,
}

impl Drop for LocalFileManager {
    fn drop(&mut self) {
        let _ =
            unsafe { Box::from_raw(self.inner_one as *mut ILocalFileManagerVTable) };
    }
}

impl ILocalFileManager for LocalFileManager {
    fn delete_local(&mut self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl IUnknown for LocalFileManager {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;
            if IsEqualGUID(riid, &IID_IUNKNOWN) {
                // Returns the nondelegating IUnknown, as in COM specification.
                *ppv = &self.non_delegating_unk as *const _ as *mut c_void;
            } else if IsEqualGUID(riid, &IID_ILOCAL_FILE_MANAGER) {
                // Returns the original VTable.
                *ppv = &self.inner_one as *const _ as *mut c_void;
            } else {
                *ppv = std::ptr::null_mut::<c_void>();
                println!("Returning NO INTERFACE.");
                return E_NOINTERFACE;
            }

            self.add_ref();
            NOERROR
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
            println!("Count is 0 for LocalFileManager. Freeing memory...");
            drop(self);
        }
        count
    }
}

// Delegating IUnknown.
unsafe extern "stdcall" fn ilocalfilemanager_query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let lfm = this as *mut LocalFileManager;
    let mut iunk_to_use : ComPtr<IUnknown> = ComPtr::new(NonNull::new((*lfm).iunk_to_use as *mut c_void).unwrap());
    let hr = iunk_to_use.query_interface(riid, ppv);
    forget(iunk_to_use);
    
    println!("Shouldn't drop here!");
    hr
}

unsafe extern "stdcall" fn ilocalfilemanager_add_ref(this: *mut IUnknownVPtr) -> u32 {
    let lfm = this as *mut LocalFileManager;
    let mut iunk_to_use : ComPtr<IUnknown> = ComPtr::new(NonNull::new((*lfm).iunk_to_use as *mut c_void).unwrap());
    let hr = iunk_to_use.add_ref();
    forget(iunk_to_use);
    
    println!("Shouldn't drop here!");
    hr
}

unsafe extern "stdcall" fn ilocalfilemanager_release(this: *mut IUnknownVPtr) -> u32 {
    let lfm = this as *mut LocalFileManager;
    let mut iunk_to_use : ComPtr<IUnknown> = ComPtr::new(NonNull::new((*lfm).iunk_to_use as *mut c_void).unwrap());
    let hr = iunk_to_use.release();
    forget(iunk_to_use);
    
    println!("Shouldn't drop here!");
    hr
}

// Non-delegating adjustor thunks.
unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = this.sub(1) as *mut LocalFileManager;
    (*this).query_interface(riid, ppv)
}

unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_add_ref(this: *mut IUnknownVPtr) -> u32 {
    let this = this.sub(1) as *mut LocalFileManager;
    (*this).add_ref()
}

unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_release(this: *mut IUnknownVPtr) -> u32 {
    let this = this.sub(1) as *mut LocalFileManager;
    (*this).release()
}

unsafe extern "stdcall" fn delete_local(this: *mut ILocalFileManagerVPtr) -> HRESULT {
    let this = this as *mut LocalFileManager;
    (*this).delete_local()
}

impl LocalFileManager {
    pub(crate) fn new(aggregate: *mut IUnknownVPtr) -> LocalFileManager {
        println!("Allocating new Vtable...");

        // Initialising the non-delegating IUnknown
        let non_del_iunknown = IUnknownMethods {
            QueryInterface: non_delegating_ilocalfilemanager_query_interface,
            Release: non_delegating_ilocalfilemanager_release,
            AddRef: non_delegating_ilocalfilemanager_add_ref,
        };

        let non_del_unknown_vptr = Box::into_raw(Box::new(IUnknownVTable(non_del_iunknown)));

        // Initialising VTable for ILocalFileManager
        let ilocalfilemanager_iunknown = IUnknownMethods {
            QueryInterface: ilocalfilemanager_query_interface,
            Release: ilocalfilemanager_release,
            AddRef: ilocalfilemanager_add_ref,
        };

        let ilocalfilemanager = ILocalFileManagerMethods {
            DeleteLocal: delete_local,
        };
        let ilocalfilemanager_vptr = Box::into_raw(Box::new(ILocalFileManagerVTable(
            ilocalfilemanager_iunknown,
            ilocalfilemanager,
        )));

        LocalFileManager {
            inner_one: ilocalfilemanager_vptr,
            non_delegating_unk: non_del_unknown_vptr,
            iunk_to_use: aggregate,
            ref_count: 0,
        }
    }
}
