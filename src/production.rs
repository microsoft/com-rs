mod class;
#[cfg(windows)]
#[doc(hidden)]
pub mod registration;

#[doc(inline)]
pub use class::{Class, ClassAllocation};
