use interface::{CLSID_LOCAL_FILE_MANAGER_CLASS, CLSID_WINDOWS_FILE_MANAGER_CLASS};

mod local_file_manager;
mod local_file_manager_class;
mod windows_file_manager;
mod windows_file_manager_class;

mod macro_test;

use local_file_manager_class::LocalFileManagerClass;
use windows_file_manager_class::WindowsFileManagerClass;

com::com_inproc_dll_module![
    (CLSID_WINDOWS_FILE_MANAGER_CLASS, WindowsFileManagerClass),
    (CLSID_LOCAL_FILE_MANAGER_CLASS, LocalFileManagerClass)
];
