use com::{
    interfaces::iunknown::IUnknown,
    runtime::{create_instance, get_class_object, init_runtime},
};
use interface::{IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

fn main() {
    init_runtime().unwrap_or_else(|hr| panic!("Failed to initialize COM Library{:x}", hr));

    let factory = get_class_object(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get cat class object 0x{:x}", hr));
    println!("Got cat class object");

    let unknown = factory
        .get_instance::<dyn IUnknown>()
        .expect("Failed to get IUnknown");
    println!("Got IUnknown");

    let animal = unknown
        .get_interface::<dyn IAnimal>()
        .expect("Failed to get IAnimal");
    println!("Got IAnimal");

    unsafe { animal.eat() };

    // Test cross-vtable interface queries for both directions.
    let domestic_animal = animal
        .get_interface::<dyn IDomesticAnimal>()
        .expect("Failed to get IDomesticAnimal");
    println!("Got IDomesticAnimal");

    unsafe { domestic_animal.train() };

    let new_cat = domestic_animal
        .get_interface::<dyn ICat>()
        .expect("Failed to get ICat");
    println!("Got ICat");
    unsafe { new_cat.ignore_humans() };

    // Test querying within second vtable.
    let domestic_animal_two = domestic_animal
        .get_interface::<dyn IDomesticAnimal>()
        .expect("Failed to get second IDomesticAnimal");
    println!("Got IDomesticAnimal");
    unsafe { domestic_animal_two.train() };

    let cat = create_instance::<dyn ICat>(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get a cat {:x}", hr));
    println!("Got another cat");

    unsafe { cat.eat() };

    assert!(animal.get_interface::<dyn ICat>().is_some());
    assert!(animal.get_interface::<dyn IUnknown>().is_some());
    assert!(animal.get_interface::<dyn IExample>().is_none());
    assert!(animal.get_interface::<dyn IDomesticAnimal>().is_some());
}
