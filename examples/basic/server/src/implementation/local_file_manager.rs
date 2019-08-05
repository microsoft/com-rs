use std::os::raw::c_void;

use crate::interface::ilocalfilemanager::{
    ILocalFileManager, ILocalFileManagerMethods, ILocalFileManagerVTable, RawILocalFileManager,
    IID_ILOCAL_FILE_MANAGER,
};
use com::{
    IID_IUnknown, IUnknownMethods, IUnknownVTable, RawIUnknown, E_NOINTERFACE, HRESULT, IID,
    LPUNKNOWN, NOERROR,
};

/// The implementation class
#[repr(C)]
pub struct LocalFileManager {
    inner_one: ILocalFileManager,
    pub non_delegating_unk: RawIUnknown,
    pub iunk_to_use: *mut RawIUnknown,
    ref_count: u32,
}

impl Drop for LocalFileManager {
    fn drop(&mut self) {
        let _ =
            unsafe { Box::from_raw(self.inner_one.inner.vtable as *mut ILocalFileManagerVTable) };
    }
}

unsafe extern "stdcall" fn ilocalfilemanager_query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let obj = this as *mut LocalFileManager;
    (*((*obj).iunk_to_use)).raw_query_interface(riid, ppv)
}

unsafe extern "stdcall" fn ilocalfilemanager_add_ref(this: *mut RawIUnknown) -> u32 {
    let obj = this as *mut LocalFileManager;

    let hr = (*((*obj).iunk_to_use)).raw_add_ref();
    hr
}

unsafe extern "stdcall" fn ilocalfilemanager_release(this: *mut RawIUnknown) -> u32 {
    let obj = this as *mut LocalFileManager;
    (*((*obj).iunk_to_use)).raw_release()
}

unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_query_interface(
    this: *mut RawIUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let obj = this.sub(1) as *mut LocalFileManager;

    match *riid {
        IID_IUnknown => {
            // Returns the nondelegating IUnknown, as in COM specification.
            *ppv = this as *mut c_void;
        }
        IID_ILOCAL_FILE_MANAGER => {
            // Returns the original VTable.
            *ppv = obj as *mut c_void;
        }
        _ => {
            *ppv = std::ptr::null_mut::<c_void>();
            println!("Returning NO INTERFACE.");
            return E_NOINTERFACE;
        }
    }

    (*this).raw_add_ref();
    NOERROR
}

unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_add_ref(this: *mut RawIUnknown) -> u32 {
    println!("Adding ref...");
    let this = this.sub(1) as *mut LocalFileManager;
    (*this).ref_count += 1;
    println!("LFM Count now {}", (*this).ref_count);
    (*this).ref_count
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_release(this: *mut RawIUnknown) -> u32 {
    println!("Releasing...");
    let this = this.sub(1) as *mut LocalFileManager;
    (*this).ref_count -= 1;
    println!("LFM Count now {}", (*this).ref_count);
    let count = (*this).ref_count;
    if count == 0 {
        println!("Count is 0. Freeing memory...");
        let _ = Box::from_raw(this);
    }
    count
}

unsafe extern "stdcall" fn delete_local(_this: *mut RawILocalFileManager) -> HRESULT {
    println!("Deleting Locally...");
    NOERROR
}

impl LocalFileManager {
    pub(crate) fn new(aggregate: *mut RawIUnknown) -> LocalFileManager {
        println!("Allocating new Vtable...");

        // Initialising the non-delegating IUnknown
        let non_del_iunknown = IUnknownMethods {
            QueryInterface: non_delegating_ilocalfilemanager_query_interface,
            Release: non_delegating_ilocalfilemanager_release,
            AddRef: non_delegating_ilocalfilemanager_add_ref,
        };

        let non_del_unknown_vtable = Box::into_raw(Box::new(IUnknownVTable(non_del_iunknown)));
        let non_del_inner = RawIUnknown {
            vtable: non_del_unknown_vtable,
        };

        // Initialising VTable for ILocalFileManager
        let ilocalfilemanager_iunknown = IUnknownMethods {
            QueryInterface: ilocalfilemanager_query_interface,
            Release: ilocalfilemanager_release,
            AddRef: ilocalfilemanager_add_ref,
        };

        let ilocalfilemanager = ILocalFileManagerMethods {
            DeleteLocal: delete_local,
        };
        let ilocalfilemanager_vtable = Box::into_raw(Box::new(ILocalFileManagerVTable(
            ilocalfilemanager_iunknown,
            ilocalfilemanager,
        )));
        let ilocalfilemanager_inner = RawILocalFileManager {
            vtable: ilocalfilemanager_vtable,
        };

        LocalFileManager {
            inner_one: ILocalFileManager {
                inner: ilocalfilemanager_inner,
            },
            non_delegating_unk: non_del_inner,
            iunk_to_use: aggregate,
            ref_count: 0,
        }
    }
}
