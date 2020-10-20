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
//!         unsafe fn eat(&self) -> com::sys::HRESULT;
//!     }
//! }
//! # fn main() {}
//! ```
//!

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

pub use com_macros::interfaces;

#[cfg(feature = "production")]
pub use com_macros::class;

// this allows for the crate to refer to itself as `com` to keep macros consistent
// whether they are used by some other crate or internally
#[doc(hidden)]
extern crate self as com;
