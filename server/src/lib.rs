mod implementation;
mod interface;

use common::{RawIUnknown, CLASS_E_CLASSNOTAVAILABLE, HRESULT, IID, LPVOID, REFCLSID, REFIID};

pub use interface::{IAnimal, ICat};

pub const CLSID_CAT_CLASS: IID = IID {
    data1: 0xC5F45CBC,
    data2: 0x4439,
    data3: 0x418C,
    data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};

#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    unsafe {
        if *rclsid != CLSID_CAT_CLASS {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        println!("Allocating new object...");
        let cat = Box::into_raw(Box::new(implementation::BritishShortHairCatClass::new()));
        (*(cat as *mut RawIUnknown)).raw_add_ref();
        let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
        (*(cat as *mut RawIUnknown)).raw_release();
        hr
    }
}
