use com::{RawIUnknown, CLASS_E_CLASSNOTAVAILABLE, HRESULT, LPVOID, REFCLSID, REFIID};
use interface::{
    CLSID_CAT_CLASS, CLSID_LOCAL_FILE_MANAGER_CLASS, CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

mod british_short_hair_cat;
mod british_short_hair_cat_class;
mod local_file_manager;
mod local_file_manager_class;
mod windows_file_manager;
mod windows_file_manager_class;

use british_short_hair_cat::BritishShortHairCat;
use british_short_hair_cat_class::BritishShortHairCatClass;
use local_file_manager::LocalFileManager;
use local_file_manager_class::LocalFileManagerClass;
use windows_file_manager::WindowsFileManager;
use windows_file_manager_class::WindowsFileManagerClass;

#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    unsafe {
        match *rclsid {
            CLSID_CAT_CLASS => {
                println!("Allocating new object CatClass...");
                let cat = Box::into_raw(Box::new(BritishShortHairCatClass::new()));
                (*(cat as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(cat as *mut RawIUnknown)).raw_release();
                hr
            }
            CLSID_WINDOWS_FILE_MANAGER_CLASS => {
                println!("Allocating new object WindowsFileManagerClass...");
                let wfm = Box::into_raw(Box::new(WindowsFileManagerClass::new()));
                (*(wfm as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(wfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(wfm as *mut RawIUnknown)).raw_release();
                hr
            }
            CLSID_LOCAL_FILE_MANAGER_CLASS => {
                println!("Allocating new object LocalFileManagerClass...");
                let lfm = Box::into_raw(Box::new(LocalFileManagerClass::new()));
                (*(lfm as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(lfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(lfm as *mut RawIUnknown)).raw_release();
                hr
            }
            _ => CLASS_E_CLASSNOTAVAILABLE,
        }
    }
}
