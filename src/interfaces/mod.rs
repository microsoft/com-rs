//! Common COM interfaces including IUnknown and IClassFactory

pub mod iclass_factory;
pub mod iunknown;

#[doc(inline)]
pub use iclass_factory::IClassFactory;
#[doc(inline)]
pub use iunknown::IUnknown;
