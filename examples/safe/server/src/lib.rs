mod clark_kent;
mod clark_kent_class;

use clark_kent_class::ClarkKentClass;
use interface::CLSID_CLARK_KENT_CLASS;

com::com_inproc_dll_module![(CLSID_CLARK_KENT_CLASS, ClarkKentClass),];
