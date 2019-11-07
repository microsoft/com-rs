# COM

[![Build Status](https://dev.azure.com/microsoft-rust/com-rs/_apis/build/status/microsoft.com-rs?branchName=master)](https://dev.azure.com/microsoft-rust/com-rs/_build/latest?definitionId=1&branchName=master)
[![Gitter](https://badges.gitter.im/com-rs/community.svg)](https://gitter.im/com-rs/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Docs.rs](https://docs.rs/com/badge.svg)](https://docs.rs/crate/com/)

A one stop shop for all things related to [COM](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal) programming in Rust.

This library exposes various macros, structs and functions to the user for both producing and consuming COM components in an idiomatic manner.

## What is COM?

> [COM](https://docs.microsoft.com/en-us/windows/win32/com/the-component-object-model) is a platform-independent, distributed, object-oriented system for creating binary software components that can interact.

COM has been superseded by [WinRT](https://docs.microsoft.com/en-us/windows/uwp/cpp-and-winrt-apis/intro-to-using-cpp-with-winrt) which builds on COM to provide even more guarantees about the binary interface. As such, if you're not sure if you need to use COM, you probably shouldn't.

## Usage

### Defining a COM interface

To both consume or produce a COM component through an interface, you will first need to generate the Rust representation of said interface. The `com_interface` macro is the main tool for automatically generating this Rust representation.

```rust
#[com_interface(00000000-0000-0000-C000-000000000046)]
pub trait IUnknown {
    unsafe fn query_interface(
        &self,
        riid: winapi::shared::guiddef::REFIID,
        ppv: *mut *mut winapi::ctypes::c_void
    ) -> winapi::shared::winerror::HRESULT;
    fn add_ref(&self) -> u32;
    unsafe fn release(&self) -> u32;
}

#[com_interface(EFF8970E-C50F-45E0-9284-291CE5A6F771)]
pub trait IAnimal: IUnknown {
    fn eat(&self) -> HRESULT;
}

```

Short explanation: This generates the VTable layout for IUnknown and implements the trait on `com::InterfaceRc` so that it dereferences the correct function pointer entry within the VTable.

### Consuming a COM component

Interaction with COM components are always through an Interface Pointer (a pointer to a pointer to a VTable). We represent such an Interface Pointer with the `com::InterfaceRc` struct, which helps manage the lifetime of the COM component through IUnknown methods.

```rust
use com::runtime::ApartmentThreadedRuntime as Runtime;

// Initialises the COM library
let runtime = Runtime::new().expect("Failed to initialize COM Library");

// Get a COM instance's interface pointer, by specifying
// - The CLSID of the COM component
// - The interface of the COM component that you want
// runtime.create_instance returns a InterfaceRc<dyn IAnimal> in this case.
let mut cat = runtime.create_instance::<dyn IAnimal>(&CLSID_CAT_CLASS).expect("Failed to get a cat");

// All IAnimal methods will be defined on InterfaceRc<T: IAnimal>
cat.eat();
```

### Producing a COM component

Producing a COM component is relatively complicated compared to consumption, due to the many features available that we must support. Here, we will walk you through producing one of our examples, the `BritishShortHairCat`.

1. Define the struct containing all the user fields you want.
- Apply the `#[co_class(...)]` macro to the struct. This will expand the struct into a COM-compatible struct, by adding COM-specific fields.
- You can then use the attribute argument `implements(...)` to indicate inheritance of any COM interfaces. The order of interfaces declared is important, as the generated vpointers are going to be in that order.

```rust
use com::co_class;

#[co_class(implements(ICat, IDomesticAnimal)]
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
- Call the provided `BritishShortHairCat::allocate()` function, passing in your user fields in the order they were declared. **IMPORTANT**
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

## Safety

While COM specifies details about the ABI of method calls, it does little in terms of guranteeing the safety of those method calls. As such, it is left up to the programmer to verify the safety of COM APIs and to write safe wrappers for those APIs.

You can read more about what gurantees this library makes in the [guide to safety](./docs/safety.md).

## Existing crates

There are many existing Rust crates that help with COM interactions. Depending on your use case, you may find these crates more suited to your needs. For example, we have
- [Intercom](https://github.com/Rantanen/intercom), which focuses on providing support for writing cross-platform COM components in Rust.
- [winapi-rs](https://github.com/retep998/winapi-rs), which provides a straightforward macro that allows you to easily consume COM interfaces.

## Notes

There are many advanced concepts in COM that our library aim to support. Relevant documentation on these advanced features can be found within the [docs](./docs) folder.

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

**Which threading models do this library support?**

As of v0.1, this library is only confident of consuming/producing COM components that live in Single-Threaded Apartments (STA). This Threading Model assumption is used in several places, so producing/consuming these COM components in a Multi-Threaded environment will not work.

**Is there out-of-process COM support?**

Currently, we only support production of in-process COM components. Also, production of a COM component can only be in the DLL format. There will be plans to enable out-of-process COM production as well as producing in the .EXE format.