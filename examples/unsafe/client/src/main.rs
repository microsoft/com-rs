// import "unknwn.idl";
// [object, uuid(DF12E151-A29A-l1dO-8C2D-00BOC73925BA)]
// interface IAnimal : IUnknown {
//   HRESULT Eat(void);
// }
// [object, uuid(DF12E152-A29A-l1dO-8C2D-0080C73925BA)]
// interface ICat : IAnimal {
//   HRESULT IgnoreHumans(void);
// }

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IID, REFCLSID, REFIID},
        minwindef::LPVOID,
        winerror::{E_FAIL, HRESULT, S_OK},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize},
        objbase::COINIT_APARTMENTTHREADED,
    },
};

use com::{
    create_instance, initialize_ex, uninitialize, ComInterface, ComPtr, IClassFactory, IUnknown,
    IID_ICLASS_FACTORY,
};
use interface::{ISuperman, CLSID_CLARK_KENT_CLASS};

fn main() {
    let result = initialize_ex();

    if let Err(hr) = result {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }

    run_safe_test();

    uninitialize();
}

fn run_safe_test() {
    let mut clark_kent = match create_instance::<ISuperman>(&CLSID_CLARK_KENT_CLASS) {
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
    let mut var_to_populate = 0u32;
    // let ptr = std::ptr::null_mut::<u32>();
    clark_kent.populate_output(&mut var_to_populate as *mut u32);
    assert!(var_to_populate == 6);

    // [in, out] tests
    let mut ptr_to_mutate = Box::into_raw(Box::new(6));
    clark_kent.mutate_and_return(ptr_to_mutate);
    assert!(unsafe { *ptr_to_mutate == 100 });

    // [in] ptr tests
    let in_var = Box::into_raw(Box::new(50));
    assert!(clark_kent.take_input_ptr(in_var) == E_FAIL);
    let in_var = Box::into_raw(Box::new(2));
    assert!(clark_kent.take_input_ptr(in_var) == S_OK);

    println!("Tests passed!");
}
