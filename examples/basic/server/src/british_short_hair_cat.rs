use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal};

use com::co_class;
use com::sys::HRESULT;

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[co_class(implements(ICat, IDomesticAnimal))]
pub struct BritishShortHairCat {
    num_owners: u32,
}

impl IDomesticAnimal for BritishShortHairCat {
    unsafe fn train(&self) -> HRESULT {
        println!("Training...");
        0
    }
}

impl ICat for BritishShortHairCat {
    unsafe fn ignore_humans(&self) -> HRESULT {
        println!("Ignoring Humans...");
        0
    }
}

impl IAnimal for BritishShortHairCat {
    unsafe fn eat(&self) -> HRESULT {
        println!("Eating...");
        0
    }
}

impl BritishShortHairCat {
    pub(crate) fn new() -> Box<BritishShortHairCat> {
        BritishShortHairCat::allocate(20)
    }
}
