use std::ffi::c_void;

pub type HRESULT = i32;
#[allow(non_snake_case)]
pub fn FAILED(result: HRESULT) -> bool {
    result < 0
}
pub type BOOL = i32;

pub const S_OK: HRESULT = 0;
pub const NOERROR: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_INVALIDARG: HRESULT = -0x7FF8_FFA9;
pub const E_NOINTERFACE: HRESULT = -0x7FFF_BFFE;
pub const E_POINTER: HRESULT = -0x7FFF_BFFD;
pub const CLASS_E_NOAGGREGATION: HRESULT = -0x7FFB_FEF0;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = -0x7FFB_FEEF;
pub const ERROR_SUCCESS: u32 = 0;
pub const SELFREG_E_CLASS: HRESULT = -0x7FFB_FDFF;
pub const COINIT_APARTMENTTHREADED: u32 = 0x2;
pub const CLSCTX_INPROC_SERVER: u32 = 0x1;

#[repr(C)]
#[derive(PartialEq)]
pub struct IID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[link(name = "ole32")]
extern "system" {
    pub fn RegCreateKeyExA(
        hKey: *mut c_void,
        lpSubKey: *const i8,
        Reserved: u32,
        lpClass: *mut u8,
        dwOptions: u32,
        samDesired: u32,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut *mut c_void,
        lpdwDisposition: *mut u32,
    ) -> i32;
    pub fn GetModuleFileNameA(hModule: *mut c_void, lpFilename: *mut i8, nSize: u32) -> u32;
    pub fn RegCloseKey(hKey: *mut c_void) -> i32;
    pub fn RegSetValueExA(
        hKey: *mut c_void,
        lpValueName: *const i8,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> i32;
    pub fn RegDeleteKeyA(hKey: *mut c_void, lpSubKey: *const i8) -> i32;
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
        pUnkOuter: *mut *const c_void,
        dwClsContext: u32,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoUninitialize();
}
