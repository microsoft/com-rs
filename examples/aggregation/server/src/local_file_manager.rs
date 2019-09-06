use com::{IUnknown, IUnknownVPtr, IUnknownVTable, iunknown_gen_vtable,};
use interface::{
    ilocal_file_manager::{
        ILocalFileManager, ILocalFileManagerVPtr, ILocalFileManagerVTable,
    },
    ilocal_file_manager_gen_vtable,
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID,},
        winerror::{E_NOINTERFACE, HRESULT, NOERROR},
    },
};

use core::mem::forget;
use com_interface_attribute::AggrCoClass;

/// The implementation class
#[repr(C)]
#[derive(AggrCoClass)]
#[com_implements(ILocalFileManager)]
pub struct InitLocalFileManager {
    user_field: u32,
}

impl ILocalFileManager for LocalFileManager {
    fn delete_local(&mut self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl LocalFileManager {
    pub(crate) fn new() -> Box<LocalFileManager> {
        let init = InitLocalFileManager {
            user_field: 2,
        };
        LocalFileManager::allocate(init)
    }
}

// ----------------------------------------- MACRO GENERATED ------------------------------------------

// #[repr(C)]
// pub struct LocalFileManager {
//     ilocalfilemanager: ILocalFileManagerVPtr,
//     non_delegating_unk: IUnknownVPtr,
//     iunk_to_use: *mut IUnknownVPtr,
//     ref_count: u32,
//     value: InitLocalFileManager,
// }

// impl Drop for LocalFileManager {
//     fn drop(&mut self) {
//         println!("Dropping LocalFileManager");
//         let _ = unsafe { Box::from_raw(self.ilocalfilemanager as *mut ILocalFileManagerVTable) };
//     }
// }

// // Default implementation should delegate to iunk_to_use.
// impl IUnknown for LocalFileManager {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         println!("Delegating QI");
//         let mut iunk_to_use: ComPtr<dyn IUnknown> = unsafe { ComPtr::new(self.iunk_to_use as *mut c_void) };
//         let hr = iunk_to_use.query_interface(riid, ppv);
//         forget(iunk_to_use);

//         hr
//     }

//     fn add_ref(&mut self) -> u32 {
//         let mut iunk_to_use: ComPtr<dyn IUnknown> = unsafe { ComPtr::new(self.iunk_to_use as *mut c_void) };
//         let res = iunk_to_use.add_ref();
//         forget(iunk_to_use);

//         res
//     }

//     fn release(&mut self) -> u32 {
//         let mut iunk_to_use: ComPtr<dyn IUnknown> = unsafe { ComPtr::new(self.iunk_to_use as *mut c_void) };
//         let res = iunk_to_use.release();
//         forget(iunk_to_use);

//         res
//     }
// }

// impl LocalFileManager {
//     fn allocate(value: InitLocalFileManager) -> Box<LocalFileManager> {
//         println!("Allocating new Vtable for LocalFileManager...");

//         // Initialising the non-delegating IUnknown
//         let non_del_iunknown = IUnknownVTable {
//             QueryInterface: non_delegating_ilocalfilemanager_query_interface,
//             Release: non_delegating_ilocalfilemanager_release,
//             AddRef: non_delegating_ilocalfilemanager_add_ref,
//         };
//         let non_del_unknown_vptr = Box::into_raw(Box::new(non_del_iunknown));

//         // Initialising VTable for ILocalFileManager
//         let ilocalfilemanager = ilocal_file_manager_gen_vtable!(LocalFileManager, 0);
//         let ilocalfilemanager_vptr = Box::into_raw(Box::new(ilocalfilemanager));

//         let out = LocalFileManager {
//             ilocalfilemanager: ilocalfilemanager_vptr,
//             non_delegating_unk: non_del_unknown_vptr,
//             iunk_to_use: std::ptr::null_mut::<IUnknownVPtr>(),
//             ref_count: 0,
//             value
//         };
//         Box::new(out)
//     }

//     // Implementations only for Aggregable objects.
//     pub(crate) fn set_iunknown(&mut self, aggr: *mut IUnknownVPtr) {
//         if aggr.is_null() {
//             self.iunk_to_use = &self.non_delegating_unk as *const _ as *mut IUnknownVPtr;
//         } else {
//             self.iunk_to_use = aggr;
//         }
//     }

//     pub(crate) fn inner_query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         println!("Non delegating QI");

//         unsafe {
//             let riid = &*riid;
//             if IsEqualGUID(riid, &IID_IUNKNOWN) {
//                 // Returns the nondelegating IUnknown, as in COM specification.
//                 *ppv = &self.non_delegating_unk as *const _ as *mut c_void;
//             } else if IsEqualGUID(riid, &IID_ILOCAL_FILE_MANAGER) {
//                 // Returns the original VTable.
//                 *ppv = &self.ilocalfilemanager as *const _ as *mut c_void;
//             } else {
//                 *ppv = std::ptr::null_mut::<c_void>();
//                 println!("Returning NO INTERFACE.");
//                 return E_NOINTERFACE;
//             }

//             self.inner_add_ref();
//             NOERROR
//         }
//     }

//     pub(crate) fn inner_add_ref(&mut self) -> u32 {
//         self.ref_count += 1;
//         println!("Count now {}", self.ref_count);
//         self.ref_count
//     }

//     pub(crate) fn inner_release(&mut self) -> u32 {
//         self.ref_count -= 1;
//         println!("Count now {}", self.ref_count);
//         let count = self.ref_count;
//         if count == 0 {
//             println!("Count is 0 for LocalFileManager. Freeing memory...");
//             drop(self);
//         }
//         count
//     }
// }

// // Non-delegating methods.
// unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_query_interface(
//     this: *mut IUnknownVPtr,
//     riid: *const IID,
//     ppv: *mut *mut c_void,
// ) -> HRESULT {
//     let this = this.sub(1) as *mut LocalFileManager;
//     (*this).inner_query_interface(riid, ppv)
// }

// unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_add_ref(
//     this: *mut IUnknownVPtr,
// ) -> u32 {
//     let this = this.sub(1) as *mut LocalFileManager;
//     (*this).inner_add_ref()
// }

// unsafe extern "stdcall" fn non_delegating_ilocalfilemanager_release(
//     this: *mut IUnknownVPtr,
// ) -> u32 {
//     let this = this.sub(1) as *mut LocalFileManager;
//     (*this).inner_release()
// }