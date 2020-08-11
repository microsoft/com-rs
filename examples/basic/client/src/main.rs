use com::{
    interfaces::iclass_factory::IClassFactory,
    interfaces::iunknown::IUnknown,
    runtime::{create_instance, get_class_object, init_apartment, ApartmentType},
};
use interface::{Food, IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

fn main() {
    init_apartment(ApartmentType::SingleThreaded)
        .unwrap_or_else(|hr| panic!("Failed to initialize COM Library{:x}", hr));
    println!("Initialized apartment");

    let factory = get_class_object::<IClassFactory>(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get cat class object 0x{:x}", hr));
    println!("Got cat class object");

    let unknown = factory
        .get_instance::<IUnknown>()
        .expect("Failed to get IUnknown");
    println!("Got IUnknown");

    let animal = unknown
        .get_interface::<IAnimal>()
        .expect("Failed to get IAnimal");
    println!("Got IAnimal");

    let food = Food { deliciousness: 10 };
    unsafe { animal.eat(&food) };
    assert!(unsafe { animal.happiness() } == 20);

    // Test cross-vtable interface queries for both directions.
    let domestic_animal = animal
        .get_interface::<IDomesticAnimal>()
        .expect("Failed to get IDomesticAnimal");
    println!("Got IDomesticAnimal");

    unsafe { domestic_animal.train() };

    let new_cat = domestic_animal
        .get_interface::<ICat>()
        .expect("Failed to get ICat");
    println!("Got ICat");
    unsafe { new_cat.ignore_humans() };

    // Test querying within second vtable.
    let domestic_animal_two = domestic_animal
        .get_interface::<IDomesticAnimal>()
        .expect("Failed to get second IDomesticAnimal");
    println!("Got IDomesticAnimal");
    unsafe { domestic_animal_two.train() };

    let cat = create_instance::<ICat>(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get a cat {:x}", hr));
    println!("Got another cat");

    unsafe { cat.eat(&food) };

    assert!(animal.get_interface::<ICat>().is_some());
    assert!(animal.get_interface::<IUnknown>().is_some());
    assert!(animal.get_interface::<IExample>().is_none());
    assert!(animal.get_interface::<IDomesticAnimal>().is_some());
}
