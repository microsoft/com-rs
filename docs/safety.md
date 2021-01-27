# Safety

COM specifies very little in the way of memory safety of COM based APIs. It is left up to the programmer to verify APIs and to make safe wrappers for them.

## The `unsafe` Keyword

It is a requirement for all usages of COM methods be marked as `unsafe`. This is required since at the time of declaring an interface, there is no way to ensure that any call to that interface will meet all of Rust's safety expectations. 

## `&self`, `&mut self`, and `self`

All methods of a COM interface are required to take an unexclusive reference to self (`&self`). This reflects the reality that COM interfaces do not have exclusive access to the underlying class,and it does not take ownership (i.e., it is not responsible for the destruction) of the underlying class. As such if you're implementing a COM server, you will most likely need to use [interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) if you would like to mutate state in method calls.

## Example

It may be helpful to look at an example of a COM interface and what code gets generated to better understand its safety properties. 

We'll try to declare a minimum COM interface. This interface will seemingly do very little, but we'll explore what the programmer must ensure for this interface to be safe.

```rust
com::interfaces! {
    #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
    pub unsafe interface IAnimal: IUnknown {
        fn Eat(&self) -> HRESULT;
    }
}
```

`IAnimal` is a minimal COM interface that only adds one method on top of the three methods defined by `IUnknown`.

This interface will expand to the following code.

```rust 
// The interface is an FFI safe struct around a non-null pointer
#[derive(Debug)]
#[repr(transparent)]
pub struct IAnimal {
    inner: std::ptr::NonNull<IAnimalVPtr>,
}

// The declared methods are generated as calls through the VTable
impl IAnimal {
    // It is up to the programmer to ensure that the pointer contained
    // in the interface is still valid.
    // This is likely to be the case as interface automatically keeps 
    // track of its reference count.
    pub unsafe fn Eat(&self) -> HRESULT {
        let interface_ptr = <Self as com::AbiTransferable>::get_abi(self);
        (interface_ptr.as_ref().as_ref().Eat)(interface_ptr)
    }
}

// All interfaces dereference to their parent interface.
impl std::ops::Deref for IAnimal {
    type Target = <IAnimal as com::Interface>::Super;
    fn deref(&self) -> &Self::Target {
        // This is safe because a valid reference to the child interface is exactly 
        // equal to a valid reference to its parent interface.
        unsafe { std::mem::transmute(self) }
    }
}

// On drop the interface will call the IUknown::Release method
impl Drop for IAnimal {
    fn drop(&mut self) {
        // This is safe because we are calling `Release` when the interface handle is no
        // longer being used.
        unsafe {
            <Self as com::Interface>::as_iunknown(self).Release();
        }
    }
}

// Cloning the interface increases its reference count.
impl ::core::clone::Clone for IAnimal {
    fn clone(&self) -> Self {
        unsafe {
            <Self as com::Interface>::as_iunknown(self).AddRef();
        }
        Self { inner: self.inner }
    }
}

// The interfaces vtable first contains the parent's vtable and then
// any methods declared on that interface
#[allow(non_snake_case)]
// Notice that this struct is declared `#[repr(C)]` as the order of the COM methods is
// essential to things working properly
#[repr(C)]
pub struct IAnimalVTable {
    pub iunknown_base: <IUnknown as com::Interface>::VTable,
    pub Eat: unsafe extern "system" fn(std::ptr::NonNull<IAnimalVPtr>) -> HRESULT,
}

pub type IAnimalVPtr = std::ptr::NonNull<IAnimalVTable>;
unsafe impl com::Interface for IAnimal {
    type VTable = IAnimalVTable;
    type Super = IUnknown;
    const IID: com::sys::IID = IID_IANIMAL;
}

pub const IID_IANIMAL: com::sys::IID = com::sys::IID {
    data1: 0xEFF8970E,
    data2: 0xC50F,
    data3: 0x45E0,
    data4: [0x92, 0x84, 0x29, 0x1C, 0xE5, 0xA6, 0xF7, 0x71],
};

/// Conversions to parent interface
impl std::convert::From<IAnimal> for IUnknown {
    fn from(this: IAnimal) -> Self {
        // This is safe because child interface handles are exactly equivalent in memory
        // to their parent interface handles
        unsafe { std::mem::transmute(this) }
    }
}
impl<'a> ::core::convert::From<&'a IAnimal> for &'a IUnknown {
    fn from(this: &'a IAnimal) -> Self {
        unsafe { ::core::mem::transmute(this) }
    }
}

// Allow this interface to be passed as a param to method which takes its parent
impl<'a> ::core::convert::Into<::com::Param<'a, IUnknown>> for IAnimal {
    fn into(self) -> ::com::Param<'a, IUnknown> {
        ::com::Param::Owned(self.into())
    }
}
impl<'a> ::core::convert::Into<::com::Param<'a, IUnknown>> for &'a IAnimal {
    fn into(self) -> ::com::Param<'a, IUnknown> {
        ::com::Param::Borrowed(self.into())
    }
}
```

## When is it safe to call a COM interface method?

First, a COM interface handle is declared as a transparent wrapper around a `std::ptr::NonNull<std::ptr::NonNull<IMyInterfaceVTable>>`. As long as interface handle was created from properly (e.g., using `com::runtime::get_class_object` to instantiate a COM class object) then the interface handle should behave correctly and should always point to a valid COM object.

The interface handle calls `IUnknown::Release` when it is dropped and `IUnknown::AddRef` when it is cloned. Proper COM class implementations should then ensure that the COM class is only released when no more interface handles exist.

COM methods must only use FFI safe arguments. Sending something without a well specified layout in memory (e.g., a Rust `Vec`) is not safe. Often times COM methods have additional properties that their arguments must meet. It is up to the programmer that the arguments meet these requirements.
