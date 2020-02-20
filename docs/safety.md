# Safety

COM specifies very little in the way of memory safety of COM based APIs. It is left up to
 the programmer to verify APIs and to make safe wrappers for them.

## The `unsafe` Keyword

It is a requirement for all methods of a `com_interface` to be marked as `unsafe`. This is required since at the time of declaring an interface, there is no way to ensure that any call to that interface will meet all of Rust's safety expectations. 

## `&self`, `&mut self`, and `self`

All methods of a `com_interface` are required to take an unexclusive reference to self 
(`&self`). This reflects the reality that COM interfaces do not have exclusive access to 
the underlying class,and it does not take ownership (i.e., it is not responsible for the 
destruction) of the underlying class. As such if you're implementing a COM server, you 
will most likely need to use [interior mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) 
if you would like to mutate state in method calls.

## Example

In order to understand the safety properties of a COM interface in Rust, it's easiest to 
look at an example. We'll try to declare a minimum COM interface. This interface will 
seemingly do very little, but we'll explore what the programmer must ensure for this
interface to be safe.

```rust
#[com_interface("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
pub trait IAnimal: IUnknown {
    unsafe fn eat(&self) -> HRESULT;
}
```

`IAnimal` is a minimal COM interface that only adds one method on top of the three methods 
defined by `IUnknown`.

This interface will expand to the following code.

```rust 
pub mod ianimal {
    use com::{com_interface, interfaces::iunknown::IUnknown, sys::HRESULT};

    // Redeclaration of the trait
    pub trait IAnimal: IUnknown {
        unsafe fn eat(&self) -> HRESULT;
    }

    // The VTable for the IAnimal interface
    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct IAnimalVTable {
        pub iunknown_base: <dyn IUnknown as com::ComInterface>::VTable,
        pub Eat: unsafe extern "stdcall" fn(*mut IAnimalVPtr) -> HRESULT,
    }

    pub type IAnimalVPtr = *const IAnimalVTable;

    // Allowing `eat` to be called on on an `ComRc<dyn IAnimal>`
    impl<T: IAnimal + com::ComInterface + ?Sized> IAnimal for com::ComRc<T> {
        unsafe fn eat(&self) -> HRESULT {
            let interface_ptr = self.as_raw() as *mut IAnimalVPtr;
            ((**interface_ptr).Eat)(interface_ptr)
        }
    }

    // Allowing `eat` to be called on on an `ComPtr<dyn IAnimal>`
    impl<T: IAnimal + com::ComInterface + ?Sized> IAnimal for com::ComPtr<T> {
        unsafe fn eat(&self) -> HRESULT {
            let interface_ptr = self.as_raw() as *mut IAnimalVPtr;
            ((**interface_ptr).Eat)(interface_ptr)
        }
    }

    // Declaration that IAnimal is a COM Interface
    unsafe impl com::ComInterface for dyn IAnimal {
        type VTable = IAnimalVTable;
        type Super = IUnknown;
        const IID: com::sys::IID = IID_IANIMAL;
        
    }

    impl<C: IAnimal> com::ProductionComInterface<C> for dyn IAnimal {
        fn vtable<O: com::offset::Offset>() -> Self::VTable {
            {
                let parent_vtable = <dyn IUnknown com::ProductionComInterface<C>>::vtable::<O>();

                // The actual real call to some `eat` COM method
                unsafe extern "stdcall" fn ianimal_eat<C: IAnimal, O: com::offset::Offset>(
                    arg0: *mut IAnimalVPtr,
                ) -> HRESULT {
                    let this = arg0.sub(O::VALUE) as *const C as *mut C;
                    (*this).eat()
                }

                IAnimalVTable {
                    iunknown_base: parent_vtable,
                    Eat: ianimal_eat::<C, O>,
                }
            }
        }
    }
    #[allow(non_upper_case_globals)]
    pub const IID_IANIMAL: com::sys::IID =
        com::sys:::IID {
            data1: 0xEFF8970E,
            data2: 0xC50F,
            data3: 0x45E0,
            data4: [0x92, 0x84, 0x29, 0x1C, 0xE5, 0xA6, 0xF7, 0x71],
        };
}
```


There's a few occasions where we're going outside of the Rust type system and relying that the 
programmer has verified everything will be ok. 

The first we should look at is where we implement `IAnimal` for `ComRc<dyn IAnimal>` and `ComPtr<dyn IAnimal>`:

```rust
impl<T: IAnimal + com::ComInterface + ?Sized> IAnimal for com::ComRc<T> {
        unsafe fn eat(&self) -> HRESULT {
            let interface_ptr = self.as_raw() as *mut IAnimalVPtr;
            ((**interface_ptr).Eat)(interface_ptr)
        }
    }

    // Allowing `eat` to be called on on an `ComPtr<dyn IAnimal>`
    impl<T: IAnimal + com::ComInterface + ?Sized> IAnimal for com::ComPtr<T> {
        unsafe fn eat(&self) -> HRESULT {
            let interface_ptr = self.as_raw() as *mut IAnimalVPtr;
            ((**interface_ptr).Eat)(interface_ptr)
        }
    }
```

Here we are casting whatever pointer the `ComRc` or `ComPtr` is holding on to
as an `*mut IAnimalVPtr` or in other words a `*mut *const IAnimalVTable`. When working 
with raw pointers there are several things we need to sure in order to verify it's safe usage
within Rust:
* The pointer is non null
* The pointer is pointing to valid data
* The pointer is not aliased for longer than the lifetime of `&self`. In other words, you can alias
the pointer but only for as long as `&self` lives. 
* The contents pointed are not mutated if there are other readers of `&self`. In other words, because we have non-exclusive access, we are not allowed to mutate the contents the pointer is pointing to. If we cannot be sure about this at compile time we should wrap this interface in an `Rc`.

The first two points should be verified in other parts of the code. The `ComRc` and `ComPtr` 
types should only be constructed if the first two points hold. If they do not hold, then some code 
somewhere else is incorrect.

The last two points cannot really be known until the point we call `ComRc<dyn IAnimal>::eat`. It 
is up to the programmer at the time of calling to ensure that these two points will hold. This is why
it the method is marked as `unsafe`. Only the programmer who is writing the code where the interface method
can be called has any possibility of verifying these rules. If they cannot be verified than the programmer
should use some runtime constructs like `Rc` to ensure this is the case. 