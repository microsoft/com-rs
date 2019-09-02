pub mod ifile_manager;
pub mod ilocal_file_manager;

pub use ifile_manager::IFileManager;
pub use ilocal_file_manager::ILocalFileManager;

use winapi::shared::guiddef::IID;

pub const CLSID_WINDOWS_FILE_MANAGER_CLASS: IID = IID {
    Data1: 0x5ffa71bd,
    Data2: 0x6d1d,
    Data3: 0x4727,
    Data4: [0xb4, 0xec, 0xda, 0x9d, 0x9d, 0x21, 0x15, 0xd1],
};

pub const CLSID_LOCAL_FILE_MANAGER_CLASS: IID = IID {
    Data1: 0xb5bbcb63,
    Data2: 0x9783,
    Data3: 0x4f96,
    Data4: [0xa0, 0x37, 0x6b, 0xb1, 0xf9, 0x8a, 0xd8, 0x44],
};
