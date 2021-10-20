//! Helpers for registering COM servers

use crate::alloc::{format, string::String, vec::Vec};
use crate::sys::{
    GetModuleFileNameA, RegCloseKey, RegCreateKeyExA, RegDeleteKeyA, RegSetValueExA, CLSID,
    ERROR_SUCCESS, FAILED, HKEY, HRESULT, LSTATUS, SELFREG_E_CLASS, S_OK,
};
extern crate std;
use core::convert::TryInto;
use core::ffi::c_void;
use core::str;
use std::ffi::CString;

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
pub fn register_keys(registry_keys_to_add: &[RegistryKeyInfo]) -> HRESULT {
    for key_info in registry_keys_to_add.iter() {
        let result = add_class_key(key_info);
        if result as u32 != ERROR_SUCCESS {
            return SELFREG_E_CLASS;
        }
    }

    S_OK
}

#[doc(hidden)]
pub fn unregister_keys(registry_keys_to_remove: &[RegistryKeyInfo]) -> HRESULT {
    let mut hr = S_OK;
    for key_info in registry_keys_to_remove.iter() {
        let result = remove_class_key(key_info);
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
    let mut hk_result = core::ptr::null_mut::<c_void>();
    let lp_class = core::ptr::null_mut::<u8>();
    let lp_security_attributes = core::ptr::null_mut::<c_void>();
    let lpdw_disposition = core::ptr::null_mut::<u32>();
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
        Err(e) => return e,
    };

    let key_handle = match set_class_key(key_handle, key_info) {
        Ok(key_handle) => key_handle,
        Err(e) => return e,
    };

    unsafe { RegCloseKey(key_handle) }
}

fn remove_class_key(key_info: &RegistryKeyInfo) -> LSTATUS {
    unsafe { RegDeleteKeyA(HKEY_CLASSES_ROOT, key_info.key_path.as_ptr()) }
}

#[doc(hidden)]
pub unsafe fn get_dll_file_path(hmodule: *mut c_void) -> String {
    const MAX_FILE_PATH_LENGTH: usize = 260;

    let mut path = [0u8; MAX_FILE_PATH_LENGTH];

    let len = GetModuleFileNameA(
        hmodule,
        path.as_mut_ptr() as *mut _,
        MAX_FILE_PATH_LENGTH as _,
    );

    String::from_utf8(path[..len as usize].to_vec()).unwrap()
}

#[doc(hidden)]
pub fn class_key_path(clsid: CLSID) -> String {
    format!("CLSID\\{{{}}}", clsid)
}

#[doc(hidden)]
pub fn class_inproc_key_path(clsid: CLSID) -> String {
    format!("CLSID\\{{{}}}\\InprocServer32", clsid)
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
        static mut _HMODULE: *mut ::core::ffi::c_void = ::core::ptr::null_mut();
        #[no_mangle]
        unsafe extern "system" fn DllMain(hinstance: *mut ::core::ffi::c_void, fdw_reason: u32, reserved: *mut ::core::ffi::c_void) -> i32 {
            const DLL_PROCESS_ATTACH: u32 = 1;
            if fdw_reason == DLL_PROCESS_ATTACH {
                unsafe { _HMODULE = hinstance; }
            }
            1
        }

        #[no_mangle]
        unsafe extern "system" fn DllGetClassObject(class_id: *const ::com::sys::CLSID, iid: *const ::com::sys::IID, result: *mut *mut ::core::ffi::c_void) -> ::com::sys::HRESULT {
            use ::com::interfaces::IUnknown;
            assert!(!class_id.is_null(), "class id passed to DllGetClassObject should never be null");

            let class_id = unsafe { &*class_id };
            if class_id == &$class_id_one {
                let instance = <$class_type_one as ::com::production::Class>::Factory::allocate();
                instance.QueryInterface(&*iid, result)
            } $(else if class_id == &$class_id {
                let instance = <$class_type_one as ::com::production::Class>::Factory::allocate();
                instance.QueryInterface(&*iid, result)
            })* else {
                ::com::sys::CLASS_E_CLASSNOTAVAILABLE
            }
        }

        #[no_mangle]
        extern "system" fn DllRegisterServer() -> ::com::sys::HRESULT {
            ::com::production::registration::dll_register_server(&mut get_relevant_registry_keys())
        }

        #[no_mangle]
        extern "system" fn DllUnregisterServer() -> ::com::sys::HRESULT {
            ::com::production::registration::dll_unregister_server(&mut get_relevant_registry_keys())
        }

        fn get_relevant_registry_keys() -> Vec<::com::production::registration::RegistryKeyInfo> {
            use ::com::production::registration::RegistryKeyInfo;
            let file_path = unsafe { ::com::production::registration::get_dll_file_path(_HMODULE) };
            vec![
                RegistryKeyInfo::new(
                    &::com::production::registration::class_key_path($class_id_one),
                    "",
                    stringify!($class_type_one),
                ),
                RegistryKeyInfo::new(
                    &::com::production::registration::class_inproc_key_path($class_id_one),
                    "",
                    &file_path,
                ),
                $(RegistryKeyInfo::new(
                    &::com::production::registration::class_key_path($class_id),
                    "",
                    stringify!($class_type),
                ),
                RegistryKeyInfo::new(
                    &::com::production::registration::class_inproc_key_path($class_id),
                    "",
                    &file_path,
                )),*
            ]
        }
    };
}
