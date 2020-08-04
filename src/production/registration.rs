//! Helpers for registering COM servers

use crate::sys::{
    GetModuleFileNameA, GetModuleHandleA, RegCloseKey, RegCreateKeyExA, RegDeleteKeyA,
    RegSetValueExA, CLSID, ERROR_SUCCESS, FAILED, GUID, HKEY, HRESULT, LSTATUS, SELFREG_E_CLASS,
    S_OK,
};

use std::convert::TryInto;
use std::ffi::c_void;
use std::ffi::CString;
use std::str;

#[doc(hidden)]
pub struct RegistryKeyInfo {
    key_path: CString,
    key_value_name: CString,
    key_value_data: CString,
}

#[doc(hidden)]
impl RegistryKeyInfo {
    pub fn new(key_path: &str, key_value_name: &str, key_value_data: &str) -> RegistryKeyInfo {
        RegistryKeyInfo {
            key_path: CString::new(key_path).unwrap(),
            key_value_name: CString::new(key_value_name).unwrap(),
            key_value_data: CString::new(key_value_data).unwrap(),
        }
    }
}

#[doc(hidden)]
pub fn register_keys(registry_keys_to_add: &Vec<RegistryKeyInfo>) -> HRESULT {
    for key_info in registry_keys_to_add.iter() {
        let result = add_class_key(&key_info);
        if result as u32 != ERROR_SUCCESS {
            return SELFREG_E_CLASS;
        }
    }

    S_OK
}

#[doc(hidden)]
pub fn unregister_keys(registry_keys_to_remove: &Vec<RegistryKeyInfo>) -> HRESULT {
    let mut hr = S_OK;
    for key_info in registry_keys_to_remove.iter() {
        let result = remove_class_key(&key_info);
        if result as u32 != ERROR_SUCCESS {
            hr = SELFREG_E_CLASS;
        }
    }

    hr
}

const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000 as HKEY;
const KEY_ALL_ACCESS: u32 = 0x000F_003F;
const REG_OPTION_NON_VOLATILE: u32 = 0x00000000;
fn create_class_key(key_info: &RegistryKeyInfo) -> Result<HKEY, LSTATUS> {
    let mut hk_result = std::ptr::null_mut::<c_void>();
    let lp_class = std::ptr::null_mut::<u8>();
    let lp_security_attributes = std::ptr::null_mut::<c_void>();
    let lpdw_disposition = std::ptr::null_mut::<u32>();
    let result = unsafe {
        RegCreateKeyExA(
            HKEY_CLASSES_ROOT,
            key_info.key_path.as_ptr(),
            0,
            lp_class,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            lp_security_attributes,
            &mut hk_result as *mut _,
            lpdw_disposition,
        )
    };
    if result as u32 != ERROR_SUCCESS {
        return Err(result);
    }

    Ok(hk_result)
}

const REG_SZ: u32 = 1;
fn set_class_key(key_handle: HKEY, key_info: &RegistryKeyInfo) -> Result<HKEY, LSTATUS> {
    let result = unsafe {
        RegSetValueExA(
            key_handle,
            key_info.key_value_name.as_ptr(),
            0,
            REG_SZ,
            key_info.key_value_data.as_ptr() as *const u8,
            key_info
                .key_value_data
                .to_bytes_with_nul()
                .len()
                .try_into()
                .unwrap(),
        )
    };
    if result as u32 != ERROR_SUCCESS {
        return Err(result);
    }

    Ok(key_handle)
}

fn add_class_key(key_info: &RegistryKeyInfo) -> LSTATUS {
    let key_handle = match create_class_key(key_info) {
        Ok(key_handle) => key_handle,
        Err(e) => {
            return e;
        }
    };

    let key_handle = match set_class_key(key_handle, key_info) {
        Ok(key_handle) => key_handle,
        Err(e) => {
            return e;
        }
    };

    unsafe { RegCloseKey(key_handle) }
}

fn remove_class_key(key_info: &RegistryKeyInfo) -> LSTATUS {
    unsafe { RegDeleteKeyA(HKEY_CLASSES_ROOT, key_info.key_path.as_ptr()) }
}

#[doc(hidden)]
pub fn get_dll_file_path() -> String {
    const MAX_FILE_PATH_LENGTH: usize = 260;

    let mut path = [0u8; MAX_FILE_PATH_LENGTH];

    let len = unsafe {
        GetModuleFileNameA(
            GetModuleHandleA(b"server.dll\0".as_ptr() as *const _),
            path.as_mut_ptr() as *mut _,
            MAX_FILE_PATH_LENGTH as _,
        )
    };

    String::from_utf8(path[..len as usize].to_vec()).unwrap()
}

#[doc(hidden)]
pub fn class_key_path(clsid: CLSID) -> String {
    format!("CLSID\\{}", guid_to_string(&clsid))
}

#[doc(hidden)]
pub fn class_inproc_key_path(clsid: CLSID) -> String {
    format!("CLSID\\{}\\InprocServer32", guid_to_string(&clsid))
}

fn guid_to_string(guid: &GUID) -> String {
    format!(
        "{{{:04X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}

/// Register the supplied keys with the registry
#[doc(hidden)]
#[inline]
pub fn dll_register_server(relevant_keys: &mut Vec<RegistryKeyInfo>) -> HRESULT {
    let hr = register_keys(relevant_keys);
    if FAILED(hr) {
        dll_unregister_server(relevant_keys);
    }

    hr
}

/// Unregister the supplied keys with the registry
#[doc(hidden)]
#[inline]
pub fn dll_unregister_server(relevant_keys: &mut Vec<RegistryKeyInfo>) -> HRESULT {
    relevant_keys.reverse();
    unregister_keys(relevant_keys)
}

/// A macro for declaring a COM server to the COM runtime
///
/// This implements the `DllGetClassObject`, `DllRegisterServer`, and `DllUnregisterServer`
/// functions on behalf of the user.
#[macro_export]
macro_rules! inproc_dll_module {
    (($class_id_one:ident, $class_type_one:ty), $(($class_id:ident, $class_type:ty)),*) => {
        #[no_mangle]
        extern "stdcall" fn DllGetClassObject(class_id: *const com::sys::CLSID, iid: *const com::sys::IID, result: *mut *mut std::ffi::c_void) -> com::sys::HRESULT {
            use com::interfaces::IUnknown;
            assert!(!class_id.is_null(), "class id passed to DllGetClassObject should never be null");

            let class_id = unsafe { &*class_id };
            if class_id == &$class_id_one {
                let mut instance = Box::new(<$class_type_one as ::com::production::Class>::Factory::new());
                let hr = unsafe {
                    instance.add_ref();
                    let hr = instance.query_interface(iid, result);
                    instance.release();
                    hr
                };
                Box::into_raw(instance);

                hr
            } $(else if class_id == &$class_id {
                let mut instance = Box::new(<$class_type_one as ::com::production::Class>::Factory::new());
                let hr = unsafe {
                    instance.add_ref();
                    let hr = instance.query_interface(iid, result);
                    instance.release();
                    hr
                };
                Box::into_raw(instance);

                hr
            })* else {
                com::sys::CLASS_E_CLASSNOTAVAILABLE
            }
        }

        #[no_mangle]
        extern "stdcall" fn DllRegisterServer() -> com::sys::HRESULT {
            com::production::registration::dll_register_server(&mut get_relevant_registry_keys())
        }

        #[no_mangle]
        extern "stdcall" fn DllUnregisterServer() -> com::sys::HRESULT {
            com::production::registration::dll_unregister_server(&mut get_relevant_registry_keys())
        }

        fn get_relevant_registry_keys() -> Vec<com::production::registration::RegistryKeyInfo> {
            use com::production::registration::RegistryKeyInfo;
            let file_path = com::production::registration::get_dll_file_path();
            vec![
                RegistryKeyInfo::new(
                    &com::production::registration::class_key_path($class_id_one),
                    "",
                    stringify!($class_type_one),
                ),
                RegistryKeyInfo::new(
                    &com::production::registration::class_inproc_key_path($class_id_one),
                    "",
                    &file_path,
                ),
                $(RegistryKeyInfo::new(
                    &com::production::registration::class_key_path($class_id),
                    "",
                    stringify!($class_type),
                ),
                RegistryKeyInfo::new(
                    &com::production::registration::class_inproc_key_path($class_id),
                    "",
                    &file_path,
                )),*
            ]
        }
    };
}
