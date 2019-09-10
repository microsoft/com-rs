use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal};

use winapi::shared::winerror::{HRESULT, NOERROR};

use com::co_class;

/// The implementation class
/// https://en.wikipedia.org/wiki/British_Shorthair
#[co_class]
#[com_implements(ICat, IDomesticAnimal)]
pub struct BritishShortHairCat {
    num_owners: u32,
}

impl IDomesticAnimal for BritishShortHairCat {
    fn train(&mut self) -> HRESULT {
        println!("Training...");
        NOERROR
    }
}

impl ICat for BritishShortHairCat {
    fn ignore_humans(&mut self) -> HRESULT {
        println!("Ignoring Humans...");
        NOERROR
    }
}

impl IAnimal for BritishShortHairCat {
    fn eat(&mut self) -> HRESULT {
        println!("Eating...");
        NOERROR
    }
}

impl BritishShortHairCat {
    pub(crate) fn new() -> Box<BritishShortHairCat> {
        BritishShortHairCat::allocate(20)
    }
}
