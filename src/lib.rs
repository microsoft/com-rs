pub mod inproc;
pub mod interfaces;
pub mod offset;
mod ptr;
mod rc;
pub mod runtime;
pub mod sys;

use interfaces::iunknown::IUnknown;
pub use ptr::ComPtr;
pub use rc::ComRc;
pub use sys::IID;

/// A COM compliant interface
///
/// # Safety
///
/// The trait or struct implementing this trait must provide a valid vtable as the
/// associated VTable type. A vtable is valid if:
/// * it is `#[repr(C)]`
/// * the type only contains `extern "stdcall" fn" definitions
pub unsafe trait ComInterface: IUnknown + 'static {
    type VTable;
    type Super: ComInterface + ?Sized;
    const IID: IID;

    fn is_iid_in_inheritance_chain(riid: &com::IID) -> bool {
        riid == &Self::IID
            || (Self::IID != <dyn IUnknown as ComInterface>::IID
                && <Self::Super as ComInterface>::is_iid_in_inheritance_chain(riid))
    }
}

/// A COM compliant class
///
/// # Safety
///
/// The implementing struct must have the following properties:
/// * it is `#[repr(C)]`
/// * The first fields of the struct are pointers to the backing VTables for
/// each of the COM Interfaces the class implements
pub unsafe trait CoClass: IUnknown {}

pub trait ProductionComInterface<T: IUnknown>: ComInterface {
    fn vtable<O: offset::Offset>() -> Self::VTable;
}

#[macro_export]
macro_rules! vtable {
    ($class:ident: $interface:ident, $offset:ident) => {
        <dyn $interface as $crate::ProductionComInterface<$class>>::vtable::<
            $crate::offset::$offset,
        >();
    };
    ($class:ident: $interface:ident, 2usize) => {
        $crate::vtable!($class: $interface, Two)
    };
    ($class:ident: $interface:ident, 1usize) => {
        $crate::vtable!($class: $interface, One)
    };
    ($class:ident: $interface:ident, 0usize) => {
        $crate::vtable!($class: $interface, Zero)
    };
    ($class:ident: $interface:ident) => {
        $crate::vtable!($class: $interface, Zero)
    };
}

#[doc(hidden)]
pub use com_macros::{co_class, com_interface, VTable};

// this allows for the crate to refer to itself as `com` to keep macros consistent
// whether they are used by some other crate or internally
#[doc(hidden)]
extern crate self as com;
