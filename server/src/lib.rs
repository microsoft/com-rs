mod implementation;
mod interface;

use common::{
    failed, RawIUnknown, CLASS_E_CLASSNOTAVAILABLE, HRESULT, IID, LPVOID, REFCLSID, REFIID,
};

pub use interface::{IAnimal, ICat};

pub const CLSID_CAT: IID = IID {
    data1: 0xC5F45CBC,
    data2: 0x4439,
    data3: 0x418C,
    data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};

#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    unsafe {
        if *rclsid != CLSID_CAT {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        println!("Allocating new object...");
        let cat = Box::into_raw(Box::new(implementation::Cat::new()));
        let hr = (*(cat as *mut RawIUnknown)).raw_query_interface(riid, ppv);
        if failed(hr) {
            println!("Querying new object failed... Deallocating object...");
            let _ = Box::from_raw(cat);
        }
        hr
    }
}
