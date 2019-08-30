use com::{create_instance, get_class_object, initialize_ex, uninitialize, IUnknown};
use interface::{IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS};

fn main() {
    let result = initialize_ex();

    if let Err(hr) = result {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }

    let result = get_class_object(&CLSID_CAT_CLASS);
    let mut factory = match result {
        Ok(factory) => factory,
        Err(hr) => {
            println!("Failed to get com class object {:x}", hr as u32);
            return;
        }
    };

    println!("Got factory.");
    let result = factory.get_instance::<dyn IUnknown>();
    let mut unknown = match result {
        Some(unknown) => unknown,
        None => {
            println!("Failed to get an unknown");
            return;
        }
    };

    let result = unknown.get_interface::<dyn IAnimal>();
    let mut animal = match result {
        Some(animal) => animal,
        None => {
            println!("Failed to get an animal");
            return;
        }
    };

    println!("Got animal.");
    animal.eat();

    // Test cross-vtable interface queries for both directions.
    let result = animal.get_interface::<dyn IDomesticAnimal>();
    let mut domestic_animal = match result {
        Some(domestic_animal) => domestic_animal,
        None => {
            println!("Failed to get domestic animal!");
            return;
        }
    };
    println!("Got domestic animal.");
    domestic_animal.train();

    let result = domestic_animal.get_interface::<dyn ICat>();
    let mut new_cat = match result {
        Some(new_cat) => new_cat,
        None => {
            println!("Failed to get domestic animal!");
            return;
        }
    };
    println!("Got domestic animal.");
    new_cat.ignore_humans();

    // Test querying within second vtable.
    let result = domestic_animal.get_interface::<dyn IDomesticAnimal>();
    let mut domestic_animal_two = match result {
        Some(domestic_animal_two) => domestic_animal_two,
        None => {
            println!("Failed to get domestic animal!");
            return;
        }
    };
    println!("Got domestic animal.");
    domestic_animal_two.train();

    // These doesn't compile
    // animal.ignore_humans();
    // animal.raw_add_ref();
    // animal.add_ref();

    let result = create_instance::<dyn ICat>(&CLSID_CAT_CLASS);
    let mut cat = match result {
        Ok(cat) => cat,
        Err(e) => {
            println!("Failed to get an cat, {:x}", e);
            return;
        }
    };
    println!("Got cat.");
    cat.eat();

    assert!(animal.get_interface::<dyn ICat>().is_some());
    assert!(animal.get_interface::<dyn IUnknown>().is_some());
    assert!(animal.get_interface::<dyn IExample>().is_none());
    assert!(animal.get_interface::<dyn IDomesticAnimal>().is_some());

    // We must drop them now or else we'll get an error when they drop after we've uninitialized COM
    drop(domestic_animal);
    drop(new_cat);
    drop(domestic_animal_two);
    drop(animal);
    drop(cat);
    drop(unknown);
    drop(factory);

    uninitialize();
}
