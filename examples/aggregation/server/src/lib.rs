use interface::{CLSID_LOCAL_FILE_MANAGER_CLASS, CLSID_WINDOWS_FILE_MANAGER_CLASS};

mod local_file_manager;
mod windows_file_manager;

use local_file_manager::LocalFileManager;
use windows_file_manager::WindowsFileManager;

com::inproc_dll_module![
    (CLSID_WINDOWS_FILE_MANAGER_CLASS, WindowsFileManager),
    (CLSID_LOCAL_FILE_MANAGER_CLASS, LocalFileManager)
];
