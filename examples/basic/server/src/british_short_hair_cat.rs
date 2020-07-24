use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal};

use com::class;
use com::sys::{HRESULT, NOERROR};

class! {
    /// The implementation class
    /// https://en.wikipedia.org/wiki/British_Shorthair
    pub class BritishShortHairCat: IDomesticAnimal(IAnimal), ICat(IAnimal) {
        num_owners: u32,
    }

    impl IDomesticAnimal for BritishShortHairCat {
        fn train(&self) -> HRESULT {
            println!("Training...");
            NOERROR
        }
    }

    impl ICat for BritishShortHairCat {
        fn ignore_humans(&self) -> HRESULT {
            println!("Ignoring Humans...");
            NOERROR
        }
    }

    impl IAnimal for BritishShortHairCat {
        fn eat(&self) -> HRESULT {
            println!("Eating...");
            NOERROR
        }
    }
}

impl Default for BritishShortHairCat {
    fn default() -> BritishShortHairCat {
        BritishShortHairCat::new(20)
    }
}
