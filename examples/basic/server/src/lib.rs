use com::{RawIUnknown, CLASS_E_CLASSNOTAVAILABLE, HRESULT, IID, LPVOID, REFCLSID, REFIID, S_OK};
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use winapi::{
    shared::{
        minwindef::{DWORD, HKEY},
        winerror::{ERROR_SUCCESS, S_FALSE},
    },
    um::{
        libloaderapi::{GetModuleFileNameA, GetModuleHandleA},
        minwinbase::SECURITY_ATTRIBUTES,
        olectl::SELFREG_E_CLASS,
        winnt::{CHAR, KEY_ALL_ACCESS, KEY_QUERY_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ},
        winreg::{
            RegCloseKey, RegCreateKeyExA, RegDeleteKeyA, RegSetValueExA, HKEY_CLASSES_ROOT, LSTATUS,
        },
    },
};

pub use interface::{IAnimal, ICat, IDomesticAnimal, IExample, IFileManager, ILocalFileManager, CLSID_CAT_CLASS, CLSID_LOCAL_FILE_MANAGER_CLASS, CLSID_WINDOWS_FILE_MANAGER_CLASS};

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

struct RegistryKeyInfo {
    key_path: CString,
    key_value_name: CString,
    key_value_data: CString,
}

impl RegistryKeyInfo {
    fn new(key_path: String, key_value_name: String, key_value_data: String) -> RegistryKeyInfo {
        RegistryKeyInfo {
            key_path: CString::new(key_path).unwrap(),
            key_value_name: CString::new(key_value_name).unwrap(),
            key_value_data: CString::new(key_value_data).unwrap(),
        }
    }
}

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

// Function tries to add ALL-OR-NONE of the registry keys.
#[no_mangle]
extern "stdcall" fn DllRegisterServer() -> HRESULT {
    unsafe {
        let registry_keys_to_add = get_relevant_registry_keys();

        for key_info in registry_keys_to_add.iter() {
            let result = add_class_key(&key_info);
            if result as u32 != ERROR_SUCCESS {
                println!("Error creating key. error code: {}", result);
                DllUnregisterServer();
                return SELFREG_E_CLASS;
            }
        }

        S_OK
    }
}

// Function tries to delete as many registry keys as possible.
#[no_mangle]
extern "stdcall" fn DllUnregisterServer() -> HRESULT {
    unsafe {
        let mut hr = S_OK;

        let mut registry_keys_to_remove = get_relevant_registry_keys();
        registry_keys_to_remove.reverse();

        for key_info in registry_keys_to_remove.iter() {
            let result = remove_class_key(&key_info);
            if result as u32 != ERROR_SUCCESS {
                println!("Error deleting key. error code: {}", result);
                hr = SELFREG_E_CLASS;
            }
        }

        hr
    }
}

fn create_class_key(key_info: &RegistryKeyInfo) -> Result<HKEY, LSTATUS> {
    let hkResult: HKEY = std::ptr::null_mut::<c_void>() as HKEY;
    let lpClass = std::ptr::null_mut::<CHAR>();
    let lpSecurityAttributes = std::ptr::null_mut::<SECURITY_ATTRIBUTES>();
    let lpdwDisposition = std::ptr::null_mut::<DWORD>();
    let result = unsafe {
        RegCreateKeyExA(
            HKEY_CLASSES_ROOT,
            key_info.key_path.as_ptr(),
            0,
            lpClass,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            lpSecurityAttributes,
            &hkResult as *const _ as *mut HKEY,
            lpdwDisposition,
        )
    };
    if result as u32 != ERROR_SUCCESS {
        return Err(result);
    }

    Ok(hkResult)
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

    unsafe {
        RegCloseKey(key_handle)
    }
}

fn remove_class_key(key_info: &RegistryKeyInfo) -> LSTATUS {
    unsafe {
        RegDeleteKeyA(HKEY_CLASSES_ROOT, key_info.key_path.as_ptr())
    }
}

fn get_dll_file_path() -> String {
    unsafe {
        let MAX_FILE_PATH_LENGTH = 260;
        let hModule = GetModuleHandleA(CString::new("server.dll").unwrap().as_ptr());
        let raw_ptr = CString::new(Vec::with_capacity(MAX_FILE_PATH_LENGTH))
            .expect("Failed to create empty string!")
            .into_raw();

        GetModuleFileNameA(hModule, raw_ptr, MAX_FILE_PATH_LENGTH.try_into().unwrap());

        CString::from_raw(raw_ptr).into_string().unwrap()
    }
}

fn class_key_path(clsid: IID) -> String {
    format!("CLSID\\{}", clsid.to_string())
}

fn class_inproc_key_path(clsid: IID) -> String {
    format!("CLSID\\{}\\InprocServer32", clsid.to_string())
}

fn get_relevant_registry_keys() -> Vec<RegistryKeyInfo> {
    let file_path = get_dll_file_path();
    // IMPORTANT: Assumption of order: Subkeys are located at a higher index than the parent key.
    vec![
        RegistryKeyInfo::new(
            class_key_path(CLSID_CAT_CLASS),
            String::from(""),
            String::from("Cat Component"),
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_CAT_CLASS),
            String::from(""),
            file_path.clone(),
        ),
        RegistryKeyInfo::new(
            class_key_path(CLSID_LOCAL_FILE_MANAGER_CLASS),
            String::from(""),
            String::from("Local File Manager Component"),
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_LOCAL_FILE_MANAGER_CLASS),
            String::from(""),
            file_path.clone(),
        ),
        RegistryKeyInfo::new(
            class_key_path(CLSID_WINDOWS_FILE_MANAGER_CLASS),
            String::from(""),
            String::from("Windows File Manager Component"),
        ),
        RegistryKeyInfo::new(
            class_inproc_key_path(CLSID_WINDOWS_FILE_MANAGER_CLASS),
            String::from(""),
            file_path.clone(),
        ),
    ]
}
