use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal, Food};

use com::class;
use com::sys::{HRESULT, NOERROR};

class! {
    /// The implementation class
    /// https://en.wikipedia.org/wiki/British_Shorthair
    pub class BritishShortHairCat: IDomesticAnimal(IAnimal), ICat(IAnimal) {
        happiness: std::cell::Cell<usize>,
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
        fn eat(&self, food: *const Food) -> HRESULT {
            assert!(!food.is_null());
            let food = unsafe { *food };
            println!("Eating...");
            self.happiness.set(self.happiness.get() + food.deliciousness);
            NOERROR
        }

        fn happiness(&self) ->usize {
            self.happiness.get()
        }
    }
}
