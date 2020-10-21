# COM

[![Build Status](https://github.com/microsoft/com-rs/workflows/Build%20and%20Test/badge.svg?event=push)](https://github.com/microsoft/com-rs/actions)
[![crates.io](https://img.shields.io/crates/v/com.svg)](https://crates.io/crates/com)
[![Docs.rs](https://docs.rs/com/badge.svg)](https://docs.rs/crate/com/)

A one stop shop for all things related to [COM](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal) programming in Rust.

This library exposes various macros, structs and functions to the user for both producing and consuming COM components in an idiomatic manner.

:rotating_light: :rotating_light: :rotating_light: **NOTE** This crate is currently in heavy development as we decide on a stable API. :rotating_light:
:rotating_light: :rotating_light:

## What is COM?

> [COM](https://docs.microsoft.com/en-us/windows/win32/com/the-component-object-model) is a platform-independent, distributed, object-oriented system for creating binary software components that can interact.

COM has been superseded by [WinRT](https://docs.microsoft.com/en-us/windows/uwp/cpp-and-winrt-apis/intro-to-using-cpp-with-winrt) which builds on COM to provide even more guarantees about the binary interface. As such, if you're not sure if you need to use COM, you probably shouldn't.

## Usage

### Defining a COM interface

To both consume or produce a COM component through an interface, you will first need to generate the Rust representation of said interface. The `interfaces` macro is the main tool for automatically generating this Rust representation.

```rust
com::interfaces! {
    #[uuid("00000000-0000-0000-C000-000000000046")]
    pub unsafe interface IUnknown {
        fn QueryInterface(
            &self,
            riid: *const IID,
            ppv: *mut *mut c_void
        ) -> HRESULT;
        fn AddRef(&self) -> u32;
        fn Release(&self) -> u32;
    }

    #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
    pub unsafe interface IAnimal: IUnknown {
        fn Eat(&self) -> HRESULT;
    }
}
```

Short explanation: This generates the VTable layout for IUnknown and IAnimal as well as the correct `Clone` and `Drop` implementations.

### Consuming a COM component

Interaction with COM components are always through an Interface Pointer (a pointer to a pointer to a VTable). 

```rust
use com::run_time::{create_instance, init_runtime};

// Initialises the COM library
init_runtime().expect("Failed to initialize COM Library");

// Get a COM instance's interface pointer, by specifying
// - The CLSID of the COM component
// - The interface of the COM component that you want
// create_instance returns an IAnimal interface in this case.
let mut cat = create_instance::<IAnimal>(&CLSID_CAT_CLASS).expect("Failed to get a cat");

// All IAnimal methods will be available.
// Because we are crossing an FFI boundary, all COM interfaces are marked as unsafe.
// It is the job of the programmer to ensure that invariants beyond what the COM library guarantees are upheld.
unsafe { cat.Eat(); }
```

For more information on usage and safety, take a look at the [docs](./docs).

### Producing a COM component

Producing a COM component is relatively complicated compared to consumption, due to the many features available that we must support. Here, we will walk you through producing one of our examples, the `BritishShortHairCat`.

1. Define the class containing all the user fields you want.
- Specify each of the interfaces the class implements. You must list the interface's parent interface in paraenthesis with the exception of IUnknown which is assumed when no parent is specified (e.g., `: MyInterface(MyParentInterface(MyGrandParentInterface))), MyOtherInterface`
2. Implement the necessary interfaces on the class.

```rust
use com::class;

com::class! {
    pub class BritishShortHairCat: ICat(IAnimal), IDomesticAnimal(IAnimal) {
        num_owners: u32,
    }
    
    impl IDomesticAnimal for BritishShortHairCat {
        fn Train(&self) -> HRESULT {
            println!("Training...");
            NOERROR
        }
    }
    
    impl ICat for BritishShortHairCat {
        fn IgnoreHumans(&self) -> HRESULT {
            println!("Ignoring Humans...");
            NOERROR
        }
    }
    
    impl IAnimal for BritishShortHairCat {
        fn Eat(&self) -> HRESULT {
            println!("Eating...");
            NOERROR
        }
    }
}
```

For more information on usage and safety, take a look at the [docs](./docs).

## Safety

While COM specifies details about the ABI of method calls, it does little in terms of guranteeing the safety of those method calls. As such, it is left up to the programmer to verify the safety of COM APIs and to write safe wrappers for those APIs.

You can read more about what gurantees this library makes in the [guide to safety](./docs/safety.md).

## Existing crates

There are many existing Rust crates that help with COM interactions. Depending on your use case, you may find these crates more suited to your needs. For example, we have
- [Intercom](https://github.com/Rantanen/intercom), which focuses on providing support for writing cross-platform COM components in Rust.
- [winapi-rs](https://github.com/retep998/winapi-rs), which provides a straightforward macro that allows you to easily consume COM interfaces.

## Building

This library is Windows only, so the easiest way to contribute will be on a Windows machine. You can execute the examples like so:

```powershell
cd examples\basic
cargo run --release
```

If you are on a Mac or Linux machine, you should still be able to make changes and check that they compile by running the following from the root of the project:

```bash
cargo check --target=x86_64-pc-windows-msvc
```
## Contributing

For further information on contributing, please take a look at the [contributing doc](./CONTRIBUTING.md)

### Code of Conduct

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct). You can find out more in the [code of conduct doc](./CODE_OF_CONDUCT.md).

## FAQ

**Is there IDL support?**

As a foundation, we are attempting to create a library that doesn't necessarily rely on having an IDL file. However, it is in the pipeline for future improvements. We will have a command-line tool that will parse the IDL into the required macros.

**Is there out-of-process COM support?**

Currently, we only support production of in-process COM components. Also, production of a COM component can only be in the DLL format. There will be plans to enable out-of-process COM production as well as producing in the .EXE format.
