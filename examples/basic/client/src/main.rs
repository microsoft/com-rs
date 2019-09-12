use com::{ApartmentThreadedRuntime as Runtime, IUnknown};
use interface::{IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

fn main() {
    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(hr) => {
            println!("Failed to initialize COM Library: {}", hr);
            return;
        }
    };

    let mut factory = match runtime.get_class_object(&CLSID_CAT_CLASS) {
        Ok(factory) => {
            println!("Got cat class object");
            factory
        }
        Err(hr) => {
            println!("Failed to get cat class object {:x}", hr as u32);
            return;
        }
    };

    let mut unknown = match factory.get_instance::<dyn IUnknown>() {
        Some(unknown) => {
            println!("Got IUnknown");
            unknown
        }
        None => {
            println!("Failed to get IUnknown");
            return;
        }
    };

    let mut animal = match unknown.get_interface::<dyn IAnimal>() {
        Some(animal) => {
            println!("Got IAnimal");
            animal
        }
        None => {
            println!("Failed to get an IAnimal");
            return;
        }
    };

    animal.eat();

    // Test cross-vtable interface queries for both directions.
    let mut domestic_animal = match animal.get_interface::<dyn IDomesticAnimal>() {
        Some(domestic_animal) => {
            println!("Got IDomesticAnimal");
            domestic_animal
        }
        None => {
            println!("Failed to get IDomesticAnimal");
            return;
        }
    };

    domestic_animal.train();

    let mut new_cat = match domestic_animal.get_interface::<dyn ICat>() {
        Some(new_cat) => {
            println!("Got ICat");
            new_cat
        }
        None => {
            println!("Failed to get ICat");
            return;
        }
    };
    new_cat.ignore_humans();

    // Test querying within second vtable.
    let mut domestic_animal_two = match domestic_animal.get_interface::<dyn IDomesticAnimal>() {
        Some(domestic_animal_two) => {
            println!("Got IDomesticAnimal");
            domestic_animal_two
        }
        None => {
            println!("Failed to get domestic animal!");
            return;
        }
    };
    domestic_animal_two.train();

    // These doesn't compile
    // animal.ignore_humans();
    // animal.raw_add_ref();
    // animal.add_ref();

    let mut cat = match runtime.create_instance::<dyn ICat>(&CLSID_CAT_CLASS) {
        Ok(cat) => {
            println!("Got another cat");
            cat
        }
        Err(e) => {
            println!("Failed to get an cat, {:x}", e);
            return;
        }
    };

    cat.eat();

    assert!(animal.get_interface::<dyn ICat>().is_some());
    assert!(animal.get_interface::<dyn IUnknown>().is_some());
    assert!(animal.get_interface::<dyn IExample>().is_none());
    assert!(animal.get_interface::<dyn IDomesticAnimal>().is_some());
}
