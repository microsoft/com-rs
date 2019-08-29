use com::{
    class_inproc_key_path, class_key_path, com_inproc_dll_module, failed, get_dll_file_path,
    register_keys, unregister_keys, IUnknown, RegistryKeyInfo,
};
use winapi::shared::{
    guiddef::{IsEqualGUID, REFCLSID, REFIID},
    minwindef::LPVOID,
    winerror::{CLASS_E_CLASSNOTAVAILABLE, HRESULT},
};

pub use interface::{
    IFileManager, ILocalFileManager, CLSID_LOCAL_FILE_MANAGER_CLASS,
    CLSID_WINDOWS_FILE_MANAGER_CLASS,
};

mod local_file_manager;
mod local_file_manager_class;
mod windows_file_manager;
mod windows_file_manager_class;

use local_file_manager::LocalFileManager;
use local_file_manager_class::LocalFileManagerClass;
use windows_file_manager::WindowsFileManager;
use windows_file_manager_class::WindowsFileManagerClass;

com_inproc_dll_module![
    (CLSID_WINDOWS_FILE_MANAGER_CLASS, WindowsFileManagerClass),
    (CLSID_LOCAL_FILE_MANAGER_CLASS, LocalFileManagerClass)
];
