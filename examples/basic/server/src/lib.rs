mod british_short_hair_cat;

use british_short_hair_cat::BritishShortHairCat;
use interface::CLSID_CAT_CLASS;

com::com_inproc_dll_module![(CLSID_CAT_CLASS, BritishShortHairCat),];
