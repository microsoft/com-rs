use crate::interfaces::iunknown::IUnknown;

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{GUID, IID, REFIID},
        minwindef::{DWORD, HKEY, LPVOID},
        winerror::{ERROR_SUCCESS, HRESULT, S_OK},
    },
    um::{
        libloaderapi::{GetModuleFileNameA, GetModuleHandleA},
        minwinbase::SECURITY_ATTRIBUTES,
        olectl::SELFREG_E_CLASS,
        winnt::{CHAR, KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ},
        winreg::{
            RegCloseKey, RegCreateKeyExA, RegDeleteKeyA, RegSetValueExA, HKEY_CLASSES_ROOT, LSTATUS,
        },
    },
};

use std::convert::TryInto;
use std::ffi::CString;

pub struct RegistryKeyInfo {
    key_path: CString,
    key_value_name: CString,
    key_value_data: CString,
}

impl RegistryKeyInfo {
    pub fn new(key_path: &str, key_value_name: &str, key_value_data: &str) -> RegistryKeyInfo {
        RegistryKeyInfo {
            key_path: CString::new(key_path).unwrap(),
            key_value_name: CString::new(key_value_name).unwrap(),
            key_value_data: CString::new(key_value_data).unwrap(),
        }
    }
}

pub fn register_keys(registry_keys_to_add: Vec<RegistryKeyInfo>) -> HRESULT {
    for key_info in registry_keys_to_add.iter() {
        let result = add_class_key(&key_info);
        if result as u32 != ERROR_SUCCESS {
            println!("Error creating key. error code: {}", result);
            return SELFREG_E_CLASS;
        }
    }

    S_OK
}

pub fn unregister_keys(registry_keys_to_remove: Vec<RegistryKeyInfo>) -> HRESULT {
    let mut hr = S_OK;
    for key_info in registry_keys_to_remove.iter() {
        let result = remove_class_key(&key_info);
        if result as u32 != ERROR_SUCCESS {
            println!("Error deleting key. error code: {}", result);
            hr = SELFREG_E_CLASS;
        }
    }

    hr
}

fn create_class_key(key_info: &RegistryKeyInfo) -> Result<HKEY, LSTATUS> {
    let hk_result: HKEY = std::ptr::null_mut::<c_void>() as HKEY;
    let lp_class = std::ptr::null_mut::<CHAR>();
    let lp_security_attributes = std::ptr::null_mut::<SECURITY_ATTRIBUTES>();
    let lpdw_disposition = std::ptr::null_mut::<DWORD>();
    let result = unsafe {
        RegCreateKeyExA(
            HKEY_CLASSES_ROOT,
            key_info.key_path.as_ptr(),
            0,
            lp_class,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            lp_security_attributes,
            &hk_result as *const _ as *mut HKEY,
            lpdw_disposition,
        )
    };
    if result as u32 != ERROR_SUCCESS {
        return Err(result);
    }

    Ok(hk_result)
}

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
            println!("Error creating key. error code: {}", e);
            return e;
        }
    };

    let key_handle = match set_class_key(key_handle, key_info) {
        Ok(key_handle) => key_handle,
        Err(e) => {
            println!("Error setting key. error code: {}", e);
            return e;
        }
    };

    unsafe { RegCloseKey(key_handle) }
}

fn remove_class_key(key_info: &RegistryKeyInfo) -> LSTATUS {
    unsafe { RegDeleteKeyA(HKEY_CLASSES_ROOT, key_info.key_path.as_ptr()) }
}

pub fn get_dll_file_path() -> String {
    unsafe {
        const MAX_FILE_PATH_LENGTH: usize = 260;
        let h_module = GetModuleHandleA(CString::new("server.dll").unwrap().as_ptr());
        let raw_ptr = CString::new(Vec::with_capacity(MAX_FILE_PATH_LENGTH))
            .expect("Failed to create empty string!")
            .into_raw();

        GetModuleFileNameA(h_module, raw_ptr, MAX_FILE_PATH_LENGTH.try_into().unwrap());

        CString::from_raw(raw_ptr).into_string().unwrap()
    }
}

pub fn class_key_path(clsid: IID) -> String {
    format!("CLSID\\{}", guid_to_string(&clsid))
}

pub fn class_inproc_key_path(clsid: IID) -> String {
    format!("CLSID\\{}\\InprocServer32", guid_to_string(&clsid))
}

fn guid_to_string(guid: &GUID) -> String {
    format!(
        "{{{:04X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        guid.Data1,
        guid.Data2,
        guid.Data3,
        guid.Data4[0],
        guid.Data4[1],
        guid.Data4[2],
        guid.Data4[3],
        guid.Data4[4],
        guid.Data4[5],
        guid.Data4[6],
        guid.Data4[7],
    )
}

#[inline]
pub fn initialize_class_object<T: IUnknown>(
    instance: Box<T>,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    instance.add_ref();
    let hr = unsafe { instance.query_interface(riid, ppv) };
    unsafe { instance.release() };
    Box::into_raw(instance);

    hr
}

#[macro_export]
macro_rules! inproc_dll_module {
    (($clsid_one:ident, $classtype_one:ty), $(($clsid:ident, $classtype:ty)),*) => {
        #[no_mangle]
        extern "stdcall" fn DllGetClassObject(rclsid: $crate::_winapi::shared::guiddef::REFCLSID, riid: $crate::_winapi::shared::guiddef::REFIID, ppv: *mut $crate::_winapi::shared::minwindef::LPVOID) -> $crate::_winapi::shared::winerror::HRESULT {
            use com::interfaces::iunknown::IUnknown;
            let rclsid = unsafe{ &*rclsid };
            if $crate::_winapi::shared::guiddef::IsEqualGUID(rclsid, &$clsid_one) {
                let mut instance = <$classtype_one>::get_class_object();
                com::inproc::initialize_class_object(instance, riid, ppv)
            } $(else if $crate::_winapi::shared::guiddef::IsEqualGUID(rclsid, &$clsid) {
                let mut instance = <$classtype>::get_class_object();
                com::inproc::initialize_class_object(instance, riid, ppv)
            })* else  {
                $crate::_winapi::shared::winerror::CLASS_E_CLASSNOTAVAILABLE
            }
        }

        #[no_mangle]
        extern "stdcall" fn DllRegisterServer() -> $crate::_winapi::shared::winerror::HRESULT {
            let hr = com::inproc::register_keys(get_relevant_registry_keys());
            if com::_winapi::shared::winerror::FAILED(hr) {
                DllUnregisterServer();
            }

            hr
        }

        #[no_mangle]
        extern "stdcall" fn DllUnregisterServer() -> $crate::_winapi::shared::winerror::HRESULT {
            let mut registry_keys_to_remove = get_relevant_registry_keys();
            registry_keys_to_remove.reverse();
            com::inproc::unregister_keys(registry_keys_to_remove)
        }


        fn get_relevant_registry_keys() -> Vec<com::inproc::RegistryKeyInfo> {
            let file_path = com::inproc::get_dll_file_path();
            vec![
                com::inproc::RegistryKeyInfo::new(
                    &com::inproc::class_key_path($clsid_one),
                    "",
                    stringify!($classtype_one),
                ),
                com::inproc::RegistryKeyInfo::new(
                    &com::inproc::class_inproc_key_path($clsid_one),
                    "",
                    &file_path,
                ),
                $(com::inproc::RegistryKeyInfo::new(
                    &com::inproc::class_key_path($clsid),
                    "",
                    stringify!($classtype),
                ),
                com::inproc::RegistryKeyInfo::new(
                    &com::inproc::class_inproc_key_path($clsid),
                    "",
                    &file_path,
                )),*
            ]
        }
    }
}
