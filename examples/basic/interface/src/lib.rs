pub mod ianimal;
pub mod icat;
pub mod icat_class;
pub mod idomesticanimal;
pub mod iexample;
pub mod ifilemanager;
pub mod ilocalfilemanager;

pub use ianimal::IAnimal;
pub use icat::ICat;
pub use icat_class::ICatClass;
pub use idomesticanimal::IDomesticAnimal;
pub use iexample::IExample;
pub use ifilemanager::IFileManager;
pub use ilocalfilemanager::ILocalFileManager;

use winapi::shared::guiddef::IID;

pub const CLSID_CAT_CLASS: IID = IID {
    Data1: 0xC5F45CBC,
    Data2: 0x4439,
    Data3: 0x418C,
    Data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};

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
