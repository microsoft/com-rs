//! Types for interacting with COM related system APIs
use core::ffi::c_void;

/// A Windows result code
pub type HRESULT = i32;

/// Equivalent of the [FAILED macro](https://docs.microsoft.com/en-us/windows/win32/api/winerror/nf-winerror-failed)
#[allow(non_snake_case)]
pub fn FAILED(result: HRESULT) -> bool {
    result < 0
}

/// BOOL type
pub type BOOL = i32;
/// LSTATUS type
pub type LSTATUS = i32;
/// HKEY type
pub type HKEY = *mut c_void;

/// No error
pub const S_OK: HRESULT = 0;
/// No error
pub const NOERROR: HRESULT = 0;
/// False
pub const S_FALSE: HRESULT = 1;

/// Argument was invalid
pub const E_INVALIDARG: HRESULT = -0x7FF8_FFA9;
/// No interface found
pub const E_NOINTERFACE: HRESULT = -0x7FFF_BFFE;
/// Invalid pointer
pub const E_POINTER: HRESULT = -0x7FFF_BFFD;

/// No aggregation for class
pub const CLASS_E_NOAGGREGATION: HRESULT = -0x7FFB_FEF0;
/// Class is not available
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = -0x7FFB_FEEF;

/// No error
pub const ERROR_SUCCESS: u32 = 0;
/// Registration error
pub const SELFREG_E_CLASS: HRESULT = -0x7FFB_FDFF;
/// A in process server
pub const CLSCTX_INPROC_SERVER: u32 = 0x1;

/// An single threaded apartment (STA)
pub const COINIT_APARTMENTTHREADED: u32 = 0x2;
/// An multi threaded apartment (STA)
pub const COINIT_MULTITHREADED: u32 = 0x0;

/// A globally unique identifier
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// An interface ID
pub type IID = GUID;
/// A class ID
pub type CLSID = GUID;

impl core::fmt::Debug for GUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
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
            self.data4[7]
        )
    }
}
impl core::fmt::Display for GUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(windows)]
#[link(name = "ole32")]
#[allow(missing_docs)]
extern "system" {
    pub fn CoIncrementMTAUsage(cookie: *mut c_void) -> HRESULT;
    pub fn RegCreateKeyExA(
        hKey: HKEY,
        lpSubKey: *const i8,
        Reserved: u32,
        lpClass: *mut u8,
        dwOptions: u32,
        samDesired: u32,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut u32,
    ) -> LSTATUS;
    pub fn GetModuleFileNameA(hModule: *mut c_void, lpFilename: *mut i8, nSize: u32) -> u32;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegSetValueExA(
        hKey: HKEY,
        lpValueName: *const i8,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> LSTATUS;
    pub fn RegDeleteKeyA(hKey: HKEY, lpSubKey: *const i8) -> LSTATUS;
    pub fn GetModuleHandleA(lpModuleName: *const i8) -> *mut c_void;
    pub fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> HRESULT;
    pub fn CoGetClassObject(
        rclsid: *const IID,
        dwClsContext: u32,
        pvReserved: *mut c_void,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoCreateInstance(
        rclsid: *const IID,
        pUnkOuter: *mut c_void,
        dwClsContext: u32,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoUninitialize();
}
