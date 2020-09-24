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

We must translate this IDL direclty into Rust code using `com::interfaces!` macro.

Take note that each method and argument must be in the same exact order and must have the same in-memory properties as the types declared in the IDL.

```rust 
/// The Data struct must match the in-memory layout exactly. We therefore use `#[repr(C)]`.
#[repr(C)]
struct Data {
    x: f32, // IDL's float type and Rust's f32 have the same in-memory layout
    y: f32, // Note, the order of the fields here matters not necessarily their names
}

com::interfaces! {
    unsafe interface IMyInterface: IUnknown {
        fn MyMethod(
            my_data: &Data,
            n: i32, // INT32 is equivalent to Rust's i32
            // This is the largest translation difference.
            // com-rs'rs interface types are equivalent to a non-null `IInterface *const` 
            // In this case, since `other` is an [out] arg, we want to be able to pass a pointer to
            // NULL which can then be set to the interface pointer. We model this by wrapping
            // the interface type in an `Option`. In other words `Option<IMyOtherInterface>` is equivalent
            // to a nullable `IIterface *const` (not the single pointer). Because we want `MyMethod` to 
            // write to `other`, we pass a `&mut`.
            other: &mut Option<IMyOtherInterface>
        ) -> com::sys::HRESULT;
    }

    unsafe interface IMyOtherInterface: IMyInterface {
        fn MyOtherMethod() -> com::sys::HRESULT;
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

TODO: document how to use COM classes