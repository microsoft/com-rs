use com::{
    class_inproc_key_path, class_key_path, failed, get_dll_file_path, register_keys,
    unregister_keys, IUnknownVPtr, RegistryKeyInfo, IUnknown
};
use std::ffi::{CStr, CString};
use winapi::shared::{
    guiddef::{IsEqualGUID, REFCLSID, REFIID},
    minwindef::LPVOID,
    winerror::{CLASS_E_CLASSNOTAVAILABLE, HRESULT},
};

pub use interface::{
    CLSID_CLARK_KENT_CLASS,
};

mod clark_kent;
mod clark_kent_class;


use clark_kent::ClarkKent;
use clark_kent_class::ClarkKentClass;

#[no_mangle]
extern "stdcall" fn DllGetClassObject(rclsid: REFCLSID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {

    unsafe {
        let rclsid = &*rclsid;
        if IsEqualGUID(rclsid, &CLSID_CLARK_KENT_CLASS) {
            println!("Allocating new object ClarkKentClass...");
            let mut ckc = Box::new(ClarkKentClass::new());
            ckc.add_ref();
            let hr = ckc.query_interface(riid, ppv);
            ckc.release();
            Box::into_raw(ckc);

            hr
        } else {
            CLASS_E_CLASSNOTAVAILABLE
        }
    }
}

// Function tries to add ALL-OR-NONE of the registry keys.
#[no_mangle]
extern "stdcall" fn DllRegisterServer() -> HRESULT {
    let hr = register_keys(get_relevant_registry_keys());
    if failed(hr) {
        DllUnregisterServer();
    }

    hr
}

// Function tries to delete as many registry keys as possible.
#[no_mangle]
extern "stdcall" fn DllUnregisterServer() -> HRESULT {
    let mut registry_keys_to_remove = get_relevant_registry_keys();
    registry_keys_to_remove.reverse();
    unregister_keys(registry_keys_to_remove)
}

fn get_relevant_registry_keys() -> Vec<RegistryKeyInfo> {
    let file_path = get_dll_file_path();
    // IMPORTANT: Assumption of order: Subkeys are located at a higher index than the parent key.
    vec![
        RegistryKeyInfo::new(
            class_key_path(CLSID_CLARK_KENT_CLASS).as_str(),
            "",
            "Clark Kent Component",
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_CLARK_KENT_CLASS).as_str(),
            "",
            file_path.clone().as_str(),
        ),
    ]
}
