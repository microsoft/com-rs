mod class;
#[cfg(windows)]
#[doc(hidden)]
#[cfg(windows)]
pub mod registration;

#[doc(inline)]
pub use class::{Class, ClassAllocation};
