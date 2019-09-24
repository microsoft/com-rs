use com::{interfaces::iunknown::IUnknown, runtime::ApartmentThreadedRuntime as Runtime};
use interface::{IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

fn main() {
    let runtime =
        Runtime::new().unwrap_or_else(|hr| panic!("Failed to initialize COM Library{:x}", hr));

    let factory = runtime
        .get_class_object(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get cat class object {:x}", hr));
    println!("Got cat class object");

    let unknown = factory
        .get_instance::<dyn IUnknown>()
        .expect("Failed to get IUknown");
    println!("Got IUnknown");

    let animal = unknown
        .get_interface::<dyn IAnimal>()
        .expect("Failed to get IAnimal");
    println!("Got IAnimal");

    animal.eat();

    // Test cross-vtable interface queries for both directions.
    let domestic_animal = animal
        .get_interface::<dyn IDomesticAnimal>()
        .expect("Failed to get IDomesticAnimal");
    println!("Got IDomesticAnimal");

    domestic_animal.train();

    let new_cat = domestic_animal
        .get_interface::<dyn ICat>()
        .expect("Failed to get ICat");
    println!("Got ICat");
    new_cat.ignore_humans();

    // Test querying within second vtable.
    let domestic_animal_two = domestic_animal
        .get_interface::<dyn IDomesticAnimal>()
        .expect("Failed to get second IDomesticAnimal");
    println!("Got IDomesticAnimal");
    domestic_animal_two.train();

    let cat = runtime
        .create_instance::<dyn ICat>(&CLSID_CAT_CLASS)
        .unwrap_or_else(|hr| panic!("Failed to get a cat {:x}", hr));
    println!("Got another cat");

    cat.eat();

    assert!(animal.get_interface::<dyn ICat>().is_some());
    assert!(animal.get_interface::<dyn IUnknown>().is_some());
    assert!(animal.get_interface::<dyn IExample>().is_none());
    assert!(animal.get_interface::<dyn IDomesticAnimal>().is_some());
}
