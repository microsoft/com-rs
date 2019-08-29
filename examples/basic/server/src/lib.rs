use com::{
    class_inproc_key_path, class_key_path, com_inproc_dll_module, failed, get_dll_file_path,
    register_keys, unregister_keys, IUnknown, RegistryKeyInfo,
};
use winapi::shared::{
    guiddef::{IsEqualGUID, REFCLSID, REFIID},
    minwindef::LPVOID,
    winerror::{CLASS_E_CLASSNOTAVAILABLE, HRESULT},
};

pub use interface::{IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

mod british_short_hair_cat;
mod british_short_hair_cat_class;

use british_short_hair_cat::BritishShortHairCat;
use british_short_hair_cat_class::BritishShortHairCatClass;

com_inproc_dll_module![(CLSID_CAT_CLASS, BritishShortHairCatClass),];
