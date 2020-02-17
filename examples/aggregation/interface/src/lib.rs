pub mod ifile_manager;
pub mod ilocal_file_manager;

pub use ifile_manager::IFileManager;
pub use ilocal_file_manager::ILocalFileManager;

use com::sys::IID;

pub const CLSID_WINDOWS_FILE_MANAGER_CLASS: IID = IID {
    data1: 0x5ffa71bd,
    data2: 0x6d1d,
    data3: 0x4727,
    data4: [0xb4, 0xec, 0xda, 0x9d, 0x9d, 0x21, 0x15, 0xd1],
};

pub const CLSID_LOCAL_FILE_MANAGER_CLASS: IID = IID {
    data1: 0xb5bbcb63,
    data2: 0x9783,
    data3: 0x4f96,
    data4: [0xa0, 0x37, 0x6b, 0xb1, 0xf9, 0x8a, 0xd8, 0x44],
};
