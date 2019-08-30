// import "unknwn.idl";
// [object, uuid(DF12E151-A29A-l1dO-8C2D-00BOC73925BA)]
// interface IAnimal : IUnknown {
//   HRESULT Eat(void);
// }
// [object, uuid(DF12E152-A29A-l1dO-8C2D-0080C73925BA)]
// interface ICat : IAnimal {
//   HRESULT IgnoreHumans(void);
// }

use com::{ComOutPtr, Runtime};
use winapi::shared::winerror::{E_FAIL, S_OK};

use interface::{ISuperman, CLSID_CLARK_KENT_CLASS};

fn main() {
    let runtime = match Runtime::new() {
        Ok(runtime) => {
            println!("Got a runtime");
            runtime
        }
        Err(hr) => {
            println!("Failed to initialize COM Library: {}", hr);
            return;
        }
    };

    run_safe_test(runtime);
}

fn run_safe_test(runtime: Runtime) {
    let mut clark_kent = match runtime.create_instance::<dyn ISuperman>(&CLSID_CLARK_KENT_CLASS) {
        Ok(clark_kent) => clark_kent,
        Err(e) => {
            println!("Failed to get clark kent, {:x}", e as u32);
            return;
        }
    };
    println!("Got clark kent!");

    // [in] tests
    assert!(clark_kent.take_input(10) == E_FAIL);
    assert!(clark_kent.take_input(4) == S_OK);

    // [out] tests
    let mut var_to_populate = ComOutPtr::<u32>::new();
    clark_kent.populate_output(&mut var_to_populate);
    assert!(*var_to_populate.get().unwrap() == 6);

    // [in, out] tests
    let mut ptr_to_mutate = Some(Box::new(6));
    clark_kent.mutate_and_return(&mut ptr_to_mutate);
    match ptr_to_mutate {
        Some(n) => assert!(*n == 100),
        None => assert!(false),
    };

    let mut ptr_to_mutate = None;
    clark_kent.mutate_and_return(&mut ptr_to_mutate);
    match ptr_to_mutate {
        Some(_n) => assert!(false),
        None => (),
    };

    // [in] ptr tests
    let in_var = Some(50);
    assert!(clark_kent.take_input_ptr(&in_var) == E_FAIL);
    let in_var = Some(2);
    assert!(clark_kent.take_input_ptr(&in_var) == S_OK);
    let in_var = None;
    assert!(clark_kent.take_input_ptr(&in_var) == E_FAIL);

    println!("Tests passed!");
}
