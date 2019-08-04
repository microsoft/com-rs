mod implementation;
mod interface;

use common::{RawIUnknown, CLASS_E_CLASSNOTAVAILABLE, HRESULT, IID, LPVOID, REFCLSID, REFIID};

pub use interface::{IAnimal, ICat, IExample, IDomesticAnimal, IFileManager, ILocalFileManager};

pub const CLSID_CAT_CLASS: IID = IID {
    data1: 0xC5F45CBC,
    data2: 0x4439,
    data3: 0x418C,
    data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};

pub const CLSID_WINDOWS_FILE_MANAGER_CLASS: IID = IID {
    data1: 0x5ffa71bd,
    data2: 0x6d1d,
    data3: 0x4727,
    data4: [0xb4, 0xec, 0xda, 0x9d, 0x9d, 0x21, 0x15, 0xd1],
};

pub const CLSID_LOCAL_FILE_MANAGER_CLASS: IID = IID {
    data1: 0xb5bbcb63,
    data2: 0x9783,
    data3: 0x4f96,
    data4: [0xa0, 0x37, 0x6b, 0xb1, 0xf9, 0x8a, 0xd8, 0x44],
};

#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    unsafe {
        match *rclsid {
            CLSID_CAT_CLASS => {
                println!("Allocating new object CatClass...");
                let cat = Box::into_raw(Box::new(implementation::BritishShortHairCatClass::new()));
                (*(cat as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(cat as *mut RawIUnknown)).raw_release();
                hr
            },
            CLSID_WINDOWS_FILE_MANAGER_CLASS => {
                println!("Allocating new object WindowsFileManagerClass...");
                let wfm = Box::into_raw(Box::new(implementation::WindowsFileManagerClass::new()));
                (*(wfm as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(wfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(wfm as *mut RawIUnknown)).raw_release();
                hr
            },
            CLSID_LOCAL_FILE_MANAGER_CLASS => {
                println!("Allocating new object LocalFileManagerClass...");
                let lfm = Box::into_raw(Box::new(implementation::LocalFileManagerClass::new()));
                (*(lfm as *mut RawIUnknown)).raw_add_ref();
                let hr = (*(lfm as *mut RawIUnknown)).raw_query_interface(riid, ppv);
                (*(lfm as *mut RawIUnknown)).raw_release();
                hr
            },
            _ => {
                CLASS_E_CLASSNOTAVAILABLE
            }
        }
        // if *rclsid != CLSID_CAT_CLASS {
        //     return CLASS_E_CLASSNOTAVAILABLE;
        // }     
    }
}
