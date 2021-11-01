use interface::{ianimal::IAnimal, icat::ICat, idomesticanimal::IDomesticAnimal, Food};

use com::class;
use com::sys::{HRESULT, NOERROR};

class! {
    /// The implementation class
    /// <https://en.wikipedia.org/wiki/British_Shorthair>
    #[derive(Debug)]
    pub class BritishShortHairCat: IDomesticAnimal(IAnimal), ICat(IAnimal) {
        happiness: std::cell::Cell<usize>,
    }

    impl IDomesticAnimal for BritishShortHairCat {
        fn Train(&self) -> HRESULT {
            println!("Training...");
            NOERROR
        }
    }

    impl ICat for BritishShortHairCat {
        fn IgnoreHumans(&self) -> HRESULT {
            println!("Ignoring Humans...");
            NOERROR
        }
    }

    impl IAnimal for BritishShortHairCat {
        fn Eat(&self, food: *const Food) -> HRESULT {
            assert!(!food.is_null());
            let food = unsafe { *food };
            println!("Eating food with deliciousness level {}...", food.deliciousness);
            self.happiness.set(self.happiness.get() + food.deliciousness);
            NOERROR
        }

        fn Happiness(&self) ->usize {
            self.happiness.get()
        }
    }
}
