//! A helper crate for consuming and producing COM interfaces.
//!
//! # Example
//!
//! To work with a COM interface it must first be declared:
//!
//! ```rust,no_run
//! /// Define an IAnimal interface
//! com::interfaces! {
//!     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
//!     pub unsafe interface IAnimal: com::interfaces::IUnknown {
//!         unsafe fn Eat(&self) -> com::sys::HRESULT;
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! To define a COM implementation class:
//!
//! ```rust,no_run
//! # com::interfaces! {
//! #     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
//! #     pub unsafe interface IAnimal: com::interfaces::IUnknown {
//! #         unsafe fn Eat(&self) -> com::sys::HRESULT;
//! #     }
//! # }
//! com::class! {
//!     pub class BritishShortHairCat: IAnimal {
//!         num_owners: u32,
//!     }
//!
//!     impl IAnimal for BritishShortHairCat {
//!         fn Eat(&self) -> com::sys::HRESULT {
//!             println!("Eating...");
//!             com::sys::NOERROR
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! See the examples directory in the repository for more examples.
//!

#![allow(clippy::transmute_ptr_to_ptr)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![deny(missing_docs)]

mod abi_transferable;
mod interface;
pub mod interfaces;
mod param;
#[cfg(windows)]
pub mod runtime;
pub mod sys;

#[cfg(feature = "production")]
/// Functionality for producing COM classes
pub mod production;

#[doc(inline)]
pub use abi_transferable::AbiTransferable;
#[doc(inline)]
pub use interface::Interface;
#[doc(inline)]
pub use param::Param;
#[doc(inline)]
pub use sys::{CLSID, IID};

/// Declare COM interfaces
///
/// # Example
/// ```rust,no_run
/// /// Define an IAnimal interface
/// com::interfaces! {
///     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
///     pub unsafe interface IAnimal: com::interfaces::IUnknown {
///         unsafe fn Eat(&self) -> com::sys::HRESULT;
///     }
/// }
/// # fn main() {}
/// ```
pub use com_macros::interfaces;

/// Declare COM implementation classes
///
/// # Example
/// ```rust,no_run
/// use com::sys::{HRESULT, NOERROR};
/// # com::interfaces! {
/// #     #[uuid("EFF8970E-C50F-45E0-9284-291CE5A6F771")]
/// #     pub unsafe interface IAnimal: com::interfaces::IUnknown {
/// #         unsafe fn Eat(&self) -> com::sys::HRESULT;
/// #     }
/// # }
///
/// com::class! {
///     pub class BritishShortHairCat: IAnimal {
///         num_owners: u32,
///     }
///
///     impl IAnimal for BritishShortHairCat {
///         fn Eat(&self) -> HRESULT {
///             println!("Eating...");
///             NOERROR
///         }
///     }
/// }
/// # fn main() {}
/// ```
#[cfg(feature = "production")]
pub use com_macros::class;

// this allows for the crate to refer to itself as `com` to keep macros consistent
// whether they are used by some other crate or internally
#[doc(hidden)]
extern crate self as com;

// We re-export `alloc` so that we can use `com::alloc::boxed::Box` in generated code,
// for code that uses `#![no_std]`.
#[doc(hidden)]
pub extern crate alloc;

/// Panics, because an `IUnknown::Release()` call has underflowed.
///
/// This function is never inlined, so it keeps the (some what verbose)
/// machinery of calling the panic handler out of mainline code, which is
/// inlined in many places. This also gives us a very convenient call stack
/// in a debugger.
///
#[doc(hidden)]
#[inline(never)]
fn release_refcount_underflow() -> ! {
    panic!("IUnknown::Release called, but refcount was zero");
}

// We check for u32::MAX / 2, instead of u32::MAX, to guard against AddRef attacks.
const REFCOUNT_OVERFLOW_MAX: u32 = u32::MAX / 2;

#[doc(hidden)]
#[inline(always)]
pub fn release_refcount(ref_count: &core::sync::atomic::AtomicU32) -> u32 {
    let old_ref_count = ref_count.fetch_sub(1, ::core::sync::atomic::Ordering::SeqCst);
    if old_ref_count == 0 {
        // The reference count was invalid.
        // In safe Rust, this should be impossible.
        // Of course, other clients outside of safe Rust can use COM.
        ::com::release_refcount_underflow();
    } else {
        old_ref_count - 1
    }
}

/// We do our best to harden code against `AddRef` attacks. An `AddRef` attack
/// is one that intentionally overflows a reference count, so that a `Release`
/// call can be used to destroy an object, even though other references are
/// still outstanding.
#[doc(hidden)]
#[inline(always)]
pub fn addref_refcount(ref_count: &core::sync::atomic::AtomicU32) -> u32 {
    let old_ref_count = ref_count.fetch_add(1, ::core::sync::atomic::Ordering::SeqCst);
    if old_ref_count >= REFCOUNT_OVERFLOW_MAX {
        // Undo the increment that we just performed.
        let _ = ref_count.fetch_sub(1, ::core::sync::atomic::Ordering::SeqCst);
        addref_overflow();
    } else {
        old_ref_count + 1
    }
}

/// Panics, because an `IUnknown::AddRef()` call has overflowed.
///
/// This function is never inlined, so it keeps the (some what verbose)
/// machinery of calling the panic handler out of mainline code, which is
/// inlined in many places. This also gives us a very convenient call stack
/// in a debugger.
#[inline(never)]
fn addref_overflow() -> ! {
    panic!("IUnknown::AddRef: refcount has overflowed");
}
