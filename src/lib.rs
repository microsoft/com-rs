// These are all defined in [winapi](https://github.com/retep998/winapi-rs)
mod comptr;

mod iclassfactory;
mod iunknown;
pub use iclassfactory::{
    IClassFactory, IClassFactoryMethods, IClassFactoryVTable, RawIClassFactory, IID_ICLASS_FACTORY,
};
pub use iunknown::{IID_IUnknown, IUnknown, IUnknownMethods, IUnknownVTable, RawIUnknown};
use std::fmt;

pub use comptr::ComPtr;
use std::os::raw::c_void;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct IID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl fmt::Display for IID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{:04X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        )
    }
}

pub type REFCLSID = *const IID;
pub type REFIID = *const IID;

pub type HRESULT = c_long;
pub fn failed(result: HRESULT) -> bool {
    result < 0
}
pub const E_NOINTERFACE: HRESULT = -0x7FFFBFFE;
pub const NOERROR: HRESULT = 0x0;
pub const S_OK: HRESULT = 0x0;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = -0x7FFBFEEF;
pub const CLASS_E_NOAGGREGATION: HRESULT = -0x7FFBFEF0;
pub const E_INVALIDARG: HRESULT = -0x7FF8FFA9;

#[allow(non_camel_case_types)]
pub type c_int = i32;
#[allow(non_camel_case_types)]
pub type c_long = i32;
#[allow(non_camel_case_types)]
pub type c_ulong = u32;
pub type LPVOID = *mut c_void;
pub type LPUNKNOWN = *mut IUnknown;
pub type DWORD = c_ulong;
pub type BOOL = c_int;

pub const COINIT_APARTMENTTHREADED: DWORD = 0x2;
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;

#[link(name = "ole32")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex
    // Initializes the COM library for use by the calling thread, sets the thread's concurrency model,
    // and creates a new apartment for the thread if one is required.
    pub fn CoInitializeEx(pvReserved: LPVOID, dwCoInit: DWORD) -> HRESULT;

    // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cogetclassobject
    // Provides a pointer to an interface on a class object associated with a specified CLSID.
    // CoGetClassObject locates, and if necessary, dynamically loads the executable code required to do this.
    pub fn CoGetClassObject(
        rclsid: REFCLSID,
        dwClsContext: DWORD,
        pvReserved: LPVOID,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;

    // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-couninitialize
    // Closes the COM library on the current thread, unloads all DLLs loaded by the thread, frees any
    // other resources that the thread maintains, and forces all RPC connections on the thread to close.
    pub fn CoUninitialize() -> ();

    // https://docs.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cocreateinstance
    pub fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: *mut RawIUnknown,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
}

/// Structs implementing this trait must have the layout of a COM Interface Pointer.
/// For example, we assume safe conversion and usage of the struct as a `RawIUnknown`.
pub unsafe trait ComInterface {
    const IID: IID;
}
