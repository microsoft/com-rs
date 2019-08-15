mod comptr;
mod iclassfactory;
mod inproc;
mod iunknown;

pub use comptr::ComPtr;
pub use iclassfactory::{
    IClassFactory, IClassFactoryMethods, IClassFactoryVTable, IID_ICLASS_FACTORY,
};
pub use inproc::*;
pub use iunknown::{IUnknown, IUnknownMethods, IUnknownVTable, IID_IUNKNOWN, IUnknownVPtr};

extern crate winapi;
use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub fn failed(result: HRESULT) -> bool {
    result < 0
}

/// Structs implementing this trait must have the layout of a COM Interface Pointer.
/// For example, we assume safe conversion and usage of the struct as a `RawIUnknown`.
pub unsafe trait ComInterface {
    const IID: IID;
}
