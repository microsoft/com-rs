mod british_short_hair_cat;
mod british_short_hair_cat_class;
mod macro_test;

use british_short_hair_cat_class::BritishShortHairCatClass;
use interface::CLSID_CAT_CLASS;

com::com_inproc_dll_module![(CLSID_CAT_CLASS, BritishShortHairCatClass),];
