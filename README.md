# COM

[![Build Status](https://dev.azure.com/microsoft-rust/com-rs/_apis/build/status/microsoft.com-rs?branchName=master)](https://dev.azure.com/microsoft-rust/com-rs/_build/latest?definitionId=1&branchName=master)

A one stop shop for all things related to [COM](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal) programming in Rust.

This library exposes various macros to the user for both producing and consuming COM components in an idiomatic manner.

# Disclaimers

**Is there IDL support?**

As a foundation, we are attempting to create a library that doesn't necessarily rely on having an IDL file. However, it is in the pipeline for future improvements. We will have a command-line tool that will parse the IDL into the required macros.

**What threading models does this library support?**

As of v0.1, this library is only capable of producing COM components that live in Single-Threaded Apartments (STA). This Threading Model assumption is used in several places, so it is Undefined Behaviour to use these COM components in a Multi-Threaded environment.

**Is there out-of-process COM support?**

Currently, we only support production of in-process COM components. Also, production of a COM component can only be in the DLL format. There will be plans to enable out-of-process COM production as well as producing in the .EXE format.

# Usage

## Defining a COM interface

To both consume or produce a COM component through an interface, you will first need to generate the Rust representation of said interface. The `com_interface` macro is the main tool for automatically generating this Rust representation.

```rust
#[com_interface(00000000-0000-0000-C000-000000000046)]
pub trait IUnknown {
    fn query_interface(
        &self,
        riid: winapi::shared::guiddef::REFIID,
        ppv: *mut *mut winapi::ctypes::c_void
    ) -> winapi::shared::winerror::HRESULT;
    fn add_ref(&self) -> u32;
    fn release(&self) -> u32;
}

#[com_interface(EFF8970E-C50F-45E0-9284-291CE5A6F771)]
pub trait IAnimal: IUnknown {
    fn eat(&self) -> HRESULT;
}

```

Short explanation: This generates the VTable layout for IUnknown and implements the trait on ComPtr so that it dereferences the correct function pointer entry within the VTable.

## Consuming a COM component

Interaction with COM components are always through an Interface Pointer (a pointer to a pointer to a VTable). We represent such an Interface Pointer with the `ComPtr` struct, which helps manage the lifetime of the COM component through IUnknown methods.

```rust
use com::Runtime;

// Initialises the COM library
let runtime = match Runtime::new() {
    Ok(runtime) => runtime,
    Err(hr) => {
        println!("Failed to initialize COM Library: {}", hr);
        return;
    }
};

// Get a COM instance's interface pointer, by specifying
// - The CLSID of the COM component
// - The interface of the COM component that you want
// runtime.create_instance returns a ComPtr<dyn IAnimal> in this case.
let mut cat = match runtime.create_instance::<dyn IAnimal>(&CLSID_CAT_CLASS) {
    Ok(cat) => cat,
    Err(e) => {
        println!("Failed to get a cat, {:x}", e);
        return;
    }
};

// All IAnimal methods will be defined on ComPtr<T: IAnimal>
cat.eat();
```

## Producing a COM component

Producing a COM component is relatively complicated compared to consumption, due to the many features available that we must support. Here, we will walk you through producing one of our examples, the `BritishShortHairCat`.

1. Define the struct containing all the user fields you want.
- Apply the `#[co_class(...)]` macro to the struct. This will expand the struct into a COM-compatible struct, by adding COM-specific fields.
- You can then use the attribute argument `com_implements(...)` to indicate inheritance of any COM interfaces. The order of interfaces declared is important, as the generated vpointers are going to be in that order.

```rust
use com::co_class;

#[co_class(com_implements(ICat, IDomesticAnimal)]
pub struct BritishShortHairCat {
    num_owners: u32,
}
```

2. Implement the necessary traits on the COM struct (in this case, `BritishShortHairCat`).

```rust
impl IDomesticAnimal for BritishShortHairCat {
    fn train(&self) -> HRESULT {
        println!("Training...");
        NOERROR
    }
}

impl ICat for BritishShortHairCat {
    fn ignore_humans(&self) -> HRESULT {
        println!("Ignoring Humans...");
        NOERROR
    }
}

impl IAnimal for BritishShortHairCat {
    fn eat(&self) -> HRESULT {
        println!("Eating...");
        NOERROR
    }
}
```

3. You will have to define a constructor with the below signature. This provides us with a standard constructor to instantiate your COM component.
```rust
fn new() -> Box<BritishShortHairCat>
```
Within this constructor, you need to
- Initialise all user fields
- Call the provided `BritishShortHairCat::allocate()` function, passing the initialised user fields **IN THE ORDER THEY WERE DECLARED**
- The `allocate` function in this case has the signature:
```rust
fn allocate(num_owners: u32) -> Box<BritishShortHairCat>
```

```rust
impl BritishShortHairCat {
    pub(crate) fn new() -> Box<BritishShortHairCat> {
        let num_owners = 20;
        BritishShortHairCat::allocate(num_owners)
    }
}
```

# Advanced COM

## Aggregation

COM allows you to aggregate other COM objects. This means exposing their interfaces as your own, allowing code reuse.

If you plan to use aggregation, then we assume you are somewhat familiar with the inner workings of COM. This explanation assumes the same.

We will walk you through producing a `WindowsFileManager`, which aggregates another COM object, the `LocalFileManager`. Specifically, we choose to aggregate the `ILocalFileManager` interface from `LocalFileManager`.

1. Define an **AGGREGABLE** com class. Here we use the `#[aggr_co_class(...)]` macro instead of the `co_class` one.

```rust
use com::aggr_co_class;

#[aggr_co_class(com_implements(ILocalFileManager)]
pub struct LocalFileManager {
    user_field_one: u32,
    user_field_two: u32,
}

impl ILocalFileManager for LocalFileManager {
    fn delete_local(&self) -> HRESULT {
        println!("Deleting Locally...");
        NOERROR
    }
}

impl LocalFileManager {
    pub(crate) fn new() -> Box<LocalFileManager> {
        let user_field_one = 20;
        let user_field_two = 40;
        LocalFileManager::allocate(user_field_one, user_field_two)
    }
}
```

2. Define the class that will aggregate `LocalFileManager`. This can be aggregable or not.
- You are responsible for instantiating your aggregates.
- In order for us to generate the correct QueryInterface implementation, you need to tell us which interfaces **EACH** aggregated object is exposing. To do this, you use a `aggr(...)` attribute argument for **EACH** aggregated object. The order in which these interfaces are defined doesn't matter.

```rust
#[co_class(com_implements(IFileManager), aggr(ILocalFileManager))]
pub struct WindowsFileManager {
    user_field_one: u32,
    user_field_two: u32,
}


impl IFileManager for WindowsFileManager {
    fn delete_all(&self) -> HRESULT {
        println!("Deleting all by delegating to Local and Remote File Managers...");
        NOERROR
    }
}
```

3. Define the class constructor. 

Here, we chose to instantiate the aggregate in the constructor. Each aggregated object is initialised as NULL, until the aggregate is instantiated, through the `set_aggregate_*` methods. Hence, you could choose to instantiate aggregates whenever you want.

In order to instantiate the aggregate, you will need to 
- Create the aggregated object as an aggregate. This can be done through CoCreateInstance,
- Supply the resultant IUnknown interface pointer to the appropriate `set_aggregate_*` methods. For each base interface exposed, there will be a separate `set_aggregate_*` method defined. Setting aggregate for one base interface will set it for every base interface exposed by the same aggregated object.

In this case, we are exposing only the `ILocalFileManager` as aggregated interfaces. This means a `set_aggregate_ilocal_file_manager` will be generated, which we can use to instantiate the underlying aggregated object. 

```rust
impl WindowsFileManager {
    pub(crate) fn new() -> Box<WindowsFileManager> {
        //Initialise the COM object.
        let user_field_one = 20;
        let user_field_two = 40;
        let mut wfm = WindowsFileManager::allocate(user_field_one, user_field_two);

        // Instantiate object to aggregate
        // TODO: Should change to use safe ComPtr methods instead.
        let mut unknown_file_manager = std::ptr::null_mut::<c_void>();
        let hr = unsafe {
            CoCreateInstance(
                &CLSID_LOCAL_FILE_MANAGER_CLASS as REFCLSID,
                &*wfm as *const _ as winapi::um::unknwnbase::LPUNKNOWN,
                CLSCTX_INPROC_SERVER,
                &IID_IUNKNOWN as REFIID,
                &mut unknown_file_manager as *mut LPVOID,
            )
        };
        if failed(hr) {
            println!("Failed to instantiate aggregate! Error: {:x}", hr as u32);
            panic!();
        }

        // Instantiate aggregate that exposes ILocalFileManager.
        wfm.set_aggregate_ilocal_file_manager(unknown_file_manager as *mut IUnknownVPtr);

        wfm
    }
}

```