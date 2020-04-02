use crate::interfaces::IUnknown;
use crate::sys::{
    GetModuleFileNameA, GetModuleHandleA, RegCloseKey, RegCreateKeyExA, RegDeleteKeyA,
    RegSetValueExA, ERROR_SUCCESS, GUID, HKEY, HRESULT, LSTATUS, SELFREG_E_CLASS, S_OK,
};

use std::convert::TryInto;
use std::ffi::c_void;
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

pub fn class_key_path(clsid: GUID) -> String {
    format!("CLSID\\{}", guid_to_string(&clsid))
}

pub fn class_inproc_key_path(clsid: GUID) -> String {
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

#[inline]
pub fn initialize_class_object<T: IUnknown>(
    instance: Box<T>,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let hr = unsafe {
        instance.add_ref();
        let hr = instance.query_interface(riid, ppv);
        instance.release();
        hr
    };
    Box::into_raw(instance);

    hr
}

#[macro_export]
#[doc(hidden)]
macro_rules! inproc_dll_module {
    (($clsid_one:ident, $classtype_one:ty), $(($clsid:ident, $classtype:ty)),*) => {
        #[no_mangle]
        extern "stdcall" fn DllGetClassObject(rclsid: *const $crate::sys::IID, riid: *const $crate::sys::IID, ppv: *mut *mut std::ffi::c_void) -> $crate::sys::HRESULT {
            use com::interfaces::iunknown::IUnknown;
            let rclsid = unsafe{ &*rclsid };
            if rclsid == &$clsid_one {
                let mut instance = <$classtype_one>::get_class_object();
                com::inproc::initialize_class_object(instance, riid, ppv)
            } $(else if rclsid == &$clsid {
                let mut instance = <$classtype>::get_class_object();
                com::inproc::initialize_class_object(instance, riid, ppv)
            })* else  {
                $crate::sys::CLASS_E_CLASSNOTAVAILABLE
            }
        }

        #[no_mangle]
        extern "stdcall" fn DllRegisterServer() -> $crate::sys::HRESULT {
            let hr = com::inproc::register_keys(get_relevant_registry_keys());
            if $crate::sys::FAILED(hr) {
                DllUnregisterServer();
            }

            hr
        }

        #[no_mangle]
        extern "stdcall" fn DllUnregisterServer() -> $crate::sys::HRESULT {
            let mut registry_keys_to_remove = get_relevant_registry_keys();
            registry_keys_to_remove.reverse();
            $crate::inproc::unregister_keys(registry_keys_to_remove)
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
    };
}
