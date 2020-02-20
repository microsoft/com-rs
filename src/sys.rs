use std::ffi::c_void;

pub type HRESULT = i32;
#[allow(non_snake_case)]
pub fn FAILED(result: HRESULT) -> bool {
    result < 0
}
pub type BOOL = i32;
pub type LSTATUS = i32;
pub type HKEY = *mut c_void;

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
#[derive(Copy, Clone, PartialEq)]
pub struct IID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl std::fmt::Debug for IID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:08X?}-{:04X?}-{:04X?}-{:02X?}{:02X?}-{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}",
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

#[link(name = "ole32")]
extern "system" {
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
