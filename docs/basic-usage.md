# Basic Usage

The following is a basic annotated example of how to use this crate.

## Interfaces

Let's assume that you want to interact with a COM API from Rust. This COM API already exists and is documented through the use of an IDL declaration.

```
typedef struct Data
{
    float x;
    float y;
} Data;

interface IMyInterface : IUnknown
{
    HRESULT MyMethod(
        const Data *my_data,
        INT32 n,
        [out] IMyOtherInterface **other
    );
}

interface IMyOtherInterface : IMyInterface
{
    HRESULT MyOtherMethod();
}
```

We must translate this IDL directly into Rust code using `com::interfaces!` macro.

Take note that each method and argument must be in the same exact order and must have the same in-memory properties as the types declared in the IDL.

```rust 
/// The Data struct must match the in-memory layout exactly. We therefore use `#[repr(C)]`.
#[repr(C)]
struct Data {
    x: f32, // IDL's float type and Rust's f32 have the same in-memory layout
    y: f32, // Note, the order of the fields here matters not necessarily their names
}

com::interfaces! {
    #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
    unsafe interface IMyInterface: IUnknown {
        fn MyMethod(
            &self,
            my_data: *const Data,
            n: i32, // INT32 is equivalent to Rust's i32
            // This is the largest translation difference.
            // com-rs'rs interface types are equivalent to a non-null `IInterface *const` 
            // In this case, since `other` is an [out] arg, we want to be able to pass a pointer to
            // NULL which can then be set to the interface pointer. We model this by wrapping
            // the interface type in an `Option`. In other words `Option<IMyOtherInterface>` is equivalent
            // to a nullable `IIterface *const` (note the single pointer). Because we want `MyMethod` to 
            // write to `other`, we pass a `*mut`.
            other: *mut Option<IMyOtherInterface>
        ) -> com::sys::HRESULT;
    }

    #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F772")]
    unsafe interface IMyOtherInterface: IMyInterface {
        fn MyOtherMethod(&self) -> com::sys::HRESULT;
    }
}
```

Using this interface is pretty straight forward. Let's assume the library you're using has a function called `getInterface` which returns a `IMyInterface *const`. This would have the following signature in Rust.

```rust
extern {
    // Note that com-rs interface types are equivalent to the C `IInterface *const`.
    fn getInterface() -> IMyInterface
}
```

To use this interface you can do the following:

```rust
let my_interface = unsafe { getInterface() };
let data = Data { x: 10.0, y: 4.0 };
let mut my_other_interface = None;
unsafe { my_interface.MyMethod(&data, 100, &mut my_other_interface) };
unsafe { my_other_interface.unwrap().MyOtherMethod() };
```

Of course, you may want to use Windows APIs for getting a registered COM component. Safe wrappers to such APIs can be found in `com::runtime`.

## Classes

Implementing COM classes is fairly straight forward. The following information is needed:
```rust
class! {
    // Make sure that the interfaces the class implements are all listed where parent interfaces
    // are specified between `()` after their child interface. If no parent is specified for an 
    // interface, it is assumed to be `IUnknown`. Multiple interface hierarchies can be specified
    // each separated by a comma.
    pub class MyCass: ISomeInterface(ISomeParentInterface(ISomeGrandparentInterface)) {
        // You can have as many inner fields as you want.
        inner_field: std::cell::Cell<usize>,
    }

    impl ISomeInterface for MyClass {
        fn SomeMethod(&self) -> HRESULT {
            NOERROR
        }
    }

    impl ISomeParentInterface for MyClass {
        fn SomeOtherMethod(&self) -> HRESULT {
            NOERROR
        }
    }

    impl ISomeGrandparentInterface for MyClass {
        fn SomeReallyOtherMethod(&self) -> HRESULT {
            NOERROR
        }
    }
}
```

Most users of class's will simply want to export that class as a COM server. Currently only declaring in-process servers are supported by com-rs. This can be done using:

```rust
com::inproc_dll_module![(CLSID_CAT_CLASS, BritishShortHairCat),];
```

This automatically exposes a `DllGetClassObject` function from the DLL that the COM runtime can use to instantiate class objects.

If you need to manually allocate your class object (e.g., when you want to return an interface pointer to a newly allocated class object from a COM method), you can allocate that class object and query for a given interface like so:

```rust
let instance = MyClass::allocate(inner_field_value);
let interface_handle = instance.query_interface::<ISomeInterface>();
```