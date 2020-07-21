use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal};

use com::co_class;
use com::sys::{HRESULT, NOERROR};

co_class! {
    /// The implementation class
    /// https://en.wikipedia.org/wiki/British_Shorthair
    pub coclass BritishShortHairCat: IDomesticAnimal(IAnimal), ICat(IAnimal) {
        num_owners: u32,
    }

    impl IDomesticAnimal for BritishShortHairCat {
        fn Train(&self) -> HRESULT {
            println!("Training...");
            NOERROR
        }
    }

    impl ICat for BritishShortHairCat {
        unsafe fn IgnoreHumans(&self) -> HRESULT {
            println!("Ignoring Humans...");
            NOERROR
        }
    }

    impl IAnimal for BritishShortHairCat {
        unsafe fn Eat(&self) -> HRESULT {
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
