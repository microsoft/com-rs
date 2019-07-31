// import "unknwn.idl";
// [object, uuid(DF12E151-A29A-l1dO-8C2D-00BOC73925BA)]
// interface IAnimal : IUnknown {
//   HRESULT Eat(void);
// }
// [object, uuid(DF12E152-A29A-l1dO-8C2D-0080C73925BA)]
// interface ICat : IAnimal {
//   HRESULT IgnoreHumans(void);
// }

use common::{
    failed, CoGetClassObject, CoInitializeEx, CoUninitialize, ComInterface, ComPtr, IID_IUnknown,
    IUnknown, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, HRESULT, IID, LPVOID, REFCLSID,
    REFIID,
};
use server::{IAnimal, ICat, CLSID_CAT};
use std::os::raw::c_void;

fn main() {
    let result = initialize_ex();

    if let Err(hr) = result {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }

    let result = get_class_object(&CLSID_CAT);
    let mut unknown = match result {
        Ok(unknown) => unknown,
        Err(hr) => {
            println!("Failed to get com class object {}", hr);
            return;
        }
    };

    println!("Got unknown.");
    let result = unknown.query_interface::<IAnimal>();
    let mut animal = match result {
        Some(animal) => animal,
        None => {
            println!("Failed to get an animal");
            return;
        }
    };

    println!("Got animal.");
    animal.eat();
    assert!(animal.query_interface::<ICat>().is_some());
    assert!(animal.query_interface::<IUnknown>().is_some());
    assert!(animal.query_interface::<IExample>().is_none());

    // These doesn't compile
    // animal.ignore_humans();
    // animal.raw_add_ref();
    // animal.add_ref();

    // We must drop them now or else we'll get an error when they drop after we've uninitialized COM
    drop(animal);
    drop(unknown);

    uninitialize();
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
fn get_class_object(iid: &IID) -> Result<ComPtr<IUnknown>, HRESULT> {
    let mut unknown = std::ptr::null_mut::<c_void>();
    let hr = unsafe {
        CoGetClassObject(
            iid as REFCLSID,
            CLSCTX_INPROC_SERVER,
            std::ptr::null_mut::<c_void>(),
            &IID_IUnknown as REFIID,
            &mut unknown as *mut LPVOID,
        )
    };
    if failed(hr) {
        return Err(hr);
    }

    Ok(unsafe { ComPtr::new(std::ptr::NonNull::new(unknown as *mut IUnknown).unwrap()) })
}

fn uninitialize() {
    unsafe { CoUninitialize() }
}

#[repr(C)]
pub struct IExample {}
pub const IID_IEXAMPLE: IID = IID {
    data1: 0xC5F45CBC,
    data2: 0x4439,
    data3: 0x418C,
    data4: [0xA9, 0xF9, 0x05, 0xAC, 0x67, 0x52, 0x5E, 0x43],
};
impl ComInterface for IExample {
    const IID: IID = IID_IEXAMPLE;
}
