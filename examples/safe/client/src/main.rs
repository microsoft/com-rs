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
        winerror::{HRESULT, S_OK, E_FAIL,},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize},
        objbase::COINIT_APARTMENTTHREADED,
    },
};

use com::{failed, ComInterface, ComPtr, IClassFactory, IUnknown, IID_ICLASS_FACTORY, ComOutPtr};
use interface::{
    CLSID_CLARK_KENT_CLASS, ISuperman,
};

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
    // let mut var_to_populate = 0u32;
    // let ptr = std::ptr::null_mut::<u32>();
    let mut var_to_populate = ComOutPtr::<u32>::new();
    clark_kent.populate_output(&mut var_to_populate);
    assert!(*var_to_populate.get().unwrap() == 6);

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

// TODO: accept threading options
fn initialize_ex() -> Result<(), HRESULT> {
    let hr = unsafe { CoInitializeEx(std::ptr::null_mut::<c_void>(), COINIT_APARTMENTTHREADED) };
    if failed(hr) {
        // TODO: https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize
        // A thread must call CoUninitialize once for each successful call it has made to the
        // CoInitialize or CoInitializeEx function, including any call that returns S_FALSE.
        return Err(hr);
    }
    Ok(())
}

// TODO: accept server options
fn get_class_object(iid: &IID) -> Result<ComPtr<IClassFactory>, HRESULT> {
    let mut class_factory = std::ptr::null_mut::<c_void>();
    let hr = unsafe {
        CoGetClassObject(
            iid as REFCLSID,
            CLSCTX_INPROC_SERVER,
            std::ptr::null_mut::<c_void>(),
            &IID_ICLASS_FACTORY as REFIID,
            &mut class_factory as *mut LPVOID,
        )
    };
    if failed(hr) {
        return Err(hr);
    }

    Ok(ComPtr::new(
        std::ptr::NonNull::new(class_factory as *mut c_void).unwrap(),
    ))
}

// TODO: accept server options
fn create_instance<T: ComInterface + ?Sized>(clsid: &IID) -> Result<ComPtr<T>, HRESULT> {
    let mut instance = std::ptr::null_mut::<c_void>();
    let hr = unsafe {
        CoCreateInstance(
            clsid as REFCLSID,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &T::IID as REFIID,
            &mut instance as *mut LPVOID,
        )
    };
    if failed(hr) {
        return Err(hr);
    }

    Ok(ComPtr::new(
        std::ptr::NonNull::new(instance as *mut c_void).unwrap(),
    ))
}

fn uninitialize() {
    unsafe { CoUninitialize() }
}
