mod comptr;
mod iclassfactory;
mod inproc;
mod iunknown;

pub use comptr::ComPtr;
pub use iclassfactory::{IClassFactory, IClassFactoryVPtr, IClassFactoryVTable, IID_ICLASSFACTORY};
pub use inproc::*;
pub use iunknown::{IUnknown, IUnknownVPtr, IUnknownVTable, IID_IUNKNOWN};

extern crate winapi;
use winapi::shared::{guiddef::IID, winerror::HRESULT};

pub fn failed(result: HRESULT) -> bool {
    result < 0
}

/// Structs implementing this trait must have the layout of a COM Interface Pointer.
/// For example, we assume safe conversion and usage of the struct as a `RawIUnknown`.
pub unsafe trait ComInterface {
    type VTable;
    const IID: IID;
}
