pub mod isuperman;

pub use isuperman::ISuperman;

use winapi::shared::guiddef::IID;

pub const CLSID_CLARK_KENT_CLASS: IID = IID {
    Data1: 0xf26c011d,
    Data2: 0xa586,
    Data3: 0x4819,
    Data4: [0xa3, 0x34, 0xa7, 0x40, 0xb4, 0xe7, 0xfd, 0x3c],
};
