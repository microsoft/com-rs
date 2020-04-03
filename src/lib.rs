//! A helper crate for consuming and producing COM interfaces.
//!
//! # Example
//!
//! To work with a COM interface it must first be declared:
//!
//! ```rust,no_run
//! /// Define an IAnimal interface wit
//! #[com::com_interface("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
//! pub trait IAnimal: com::interfaces::IUnknown {
//!     unsafe fn eat(&self) -> com::sys::HRESULT;
//! }
//! ```
//!

#![deny(missing_docs)]

pub mod interfaces;
#[doc(hidden)]
pub mod offset;
mod ptr;
mod rc;
#[doc(hidden)]
pub mod registration;
pub mod runtime;
pub mod sys;

use interfaces::IUnknown;
pub use ptr::ComPtr;
pub use rc::ComRc;
#[doc(inline)]
pub use sys::{CLSID, IID};

/// A COM compliant interface
///
/// # Safety
///
/// The trait or struct implementing this trait must provide a valid vtable as the
/// associated VTable type. A vtable is valid if:
/// * it is `#[repr(C)]`
/// * the type only contains `extern "stdcall" fn" definitions
pub unsafe trait ComInterface: IUnknown + 'static {
    /// A COM compatible V-Table
    type VTable;
    /// The interface that this interface inherits from
    type Super: ComInterface + ?Sized;
    /// The associated id for this interface
    const IID: IID;

    /// Check whether a given IID is in the inheritance hierarchy of this interface
    fn is_iid_in_inheritance_chain(riid: &IID) -> bool {
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

/// A COM interface that will be exposed in a COM server
pub trait ProductionComInterface<T: IUnknown>: ComInterface {
    /// Get the vtable for a particular COM interface
    fn vtable<O: offset::Offset>() -> Self::VTable;
}

#[doc(hidden)]
#[macro_export]
macro_rules! vtable {
    ($class:ident: $interface:ident, $offset:ident) => {
        <dyn $interface as $crate::ProductionComInterface<$class>>::vtable::<
            $crate::offset::$offset,
        >();
    };
    ($class:ident: $interface:ident, 4usize) => {
        $crate::vtable!($class: $interface, Four)
    };
    ($class:ident: $interface:ident, 3usize) => {
        $crate::vtable!($class: $interface, Three)
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
