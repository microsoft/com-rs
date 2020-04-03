//! Common COM interfaces including IUknown and IClassFactory

pub mod iclass_factory;
pub mod iunknown;

pub use iclass_factory::IClassFactory;
pub use iunknown::IUnknown;
