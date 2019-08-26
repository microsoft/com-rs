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
        winerror::HRESULT,
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoCreateInstance, CoGetClassObject, CoInitializeEx, CoUninitialize},
        objbase::COINIT_APARTMENTTHREADED,
    },
};

use com::{failed, ComInterface, ComPtr, IClassFactory, IUnknown, IID_ICLASSFACTORY};
use interface::{
    IAnimal, ICat, IDomesticAnimal, IExample, CLSID_CAT_CLASS,
    CLSID_LOCAL_FILE_MANAGER_CLASS, ILocalFileManager,
    CLSID_WINDOWS_FILE_MANAGER_CLASS, IFileManager,
};

fn main() {
    let result = initialize_ex();

    if let Err(hr) = result {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }

    run_aggr_test();

    let result = get_class_object(&CLSID_CAT_CLASS);
    let mut factory = match result {
        Ok(factory) => factory,
        Err(hr) => {
            println!("Failed to get com class object {:x}", hr as u32);
            return;
        }
    };

    println!("Got factory.");
    let result = factory.get_instance::<IUnknown>();
    let mut unknown = match result {
        Some(unknown) => unknown,
        None => {
            println!("Failed to get an unknown");
            return;
        }
    };

    let result = unknown.get_interface::<IAnimal>();
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
    let result = animal.get_interface::<IDomesticAnimal>();
    let mut domestic_animal = match result {
        Some(domestic_animal) => domestic_animal,
        None => {
            println!("Failed to get domestic animal!");
            return;
        }
    };
    println!("Got domestic animal.");
    domestic_animal.train();

    let result = domestic_animal.get_interface::<ICat>();
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
    let result = domestic_animal.get_interface::<IDomesticAnimal>();
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

    let result = create_instance::<ICat>(&CLSID_CAT_CLASS);
    let mut cat = match result {
        Ok(cat) => cat,
        Err(e) => {
            println!("Failed to get an cat, {:x}", e as u32);
            return;
        }
    };
    println!("Got cat.");
    cat.eat();

    assert!(animal.get_interface::<ICat>().is_some());
    assert!(animal.get_interface::<IUnknown>().is_some());
    assert!(animal.get_interface::<IExample>().is_none());
    assert!(animal.get_interface::<IDomesticAnimal>().is_some());

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

fn run_aggr_test() {
    let result = create_instance::<IFileManager>(&CLSID_WINDOWS_FILE_MANAGER_CLASS);
    let mut filemanager = match result {
        Ok(filemanager) => filemanager,
        Err(e) => {
            println!("Failed to get filemanager, {:x}", e as u32);
            return;
        }
    };
    println!("Got filemanager!");
    filemanager.delete_all();

    let result = filemanager.get_interface::<ILocalFileManager>();
    let mut lfm = match result {
        Some(lfm) => lfm,
        None => {
            println!("Failed to get Local File Manager!");
            return;
        }
    };
    println!("Got Local File Manager.");
    lfm.delete_local();

    let result = create_instance::<ILocalFileManager>(&CLSID_LOCAL_FILE_MANAGER_CLASS);
    let mut localfilemanager = match result {
        Ok(localfilemanager) => localfilemanager,
        Err(e) => {
            println!("Failed to get localfilemanager, {:x}", e as u32);
            return;
        }
    };
    println!("Got localfilemanager!");
    localfilemanager.delete_local();
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
            &IID_ICLASSFACTORY as REFIID,
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
