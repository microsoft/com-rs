pub mod ianimal;
pub mod icat;
pub mod icat_class;
pub mod idomesticanimal;
pub mod iexample;

pub use ianimal::IAnimal;
pub use icat::ICat;
pub use icat_class::ICatClass;
pub use idomesticanimal::IDomesticAnimal;
pub use iexample::IExample;

use winapi::shared::guiddef::IID;

pub const CLSID_CAT_CLASS: IID = IID {
    Data1: 0xC5F45CBC,
    Data2: 0x4439,
    Data3: 0x418C,
    Data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};
