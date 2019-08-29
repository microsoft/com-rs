use com::com_inproc_dll_module;

pub use interface::CLSID_CAT_CLASS;

mod british_short_hair_cat;
mod british_short_hair_cat_class;

use british_short_hair_cat::BritishShortHairCat;
use british_short_hair_cat_class::BritishShortHairCatClass;

com_inproc_dll_module![(CLSID_CAT_CLASS, BritishShortHairCatClass),];
