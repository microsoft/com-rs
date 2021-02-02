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

#[doc(hidden)]
pub type RawPtr = *mut std::ffi::c_void;

#[doc(hidden)]
pub use const_sha1::ConstBuffer;

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

#[doc(hidden)]
pub trait HexReader {
    fn next_u8(&mut self) -> u8;
    fn next_u16(&mut self) -> u16;
    fn next_u32(&mut self) -> u32;
}

impl HexReader for std::str::Bytes<'_> {
    fn next_u8(&mut self) -> u8 {
        let value = self.next().unwrap();
        match value {
            b'0'..=b'9' => value - b'0',
            b'A'..=b'F' => 10 + value - b'A',
            b'a'..=b'f' => 10 + value - b'a',
            _ => panic!("Invalid GUID string"),
        }
    }

    fn next_u16(&mut self) -> u16 {
        self.next_u8().into()
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u8().into()
    }
}


impl From<&str> for GUID {
    fn from(value: &str) -> GUID {
        assert!(value.len() == 36, "Invalid GUID string");
        let mut bytes = value.bytes();

        let a = ((bytes.next_u32() * 16 + bytes.next_u32()) << 24)
            + ((bytes.next_u32() * 16 + bytes.next_u32()) << 16)
            + ((bytes.next_u32() * 16 + bytes.next_u32()) << 8)
            + bytes.next_u32() * 16
            + bytes.next_u32();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let b = ((bytes.next_u16() * 16 + (bytes.next_u16())) << 8)
            + bytes.next_u16() * 16
            + bytes.next_u16();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let c = ((bytes.next_u16() * 16 + bytes.next_u16()) << 8)
            + bytes.next_u16() * 16
            + bytes.next_u16();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");
        let d = bytes.next_u8() * 16 + bytes.next_u8();
        let e = bytes.next_u8() * 16 + bytes.next_u8();
        assert!(bytes.next().unwrap() == b'-', "Invalid GUID string");

        let f = bytes.next_u8() * 16 + bytes.next_u8();
        let g = bytes.next_u8() * 16 + bytes.next_u8();
        let h = bytes.next_u8() * 16 + bytes.next_u8();
        let i = bytes.next_u8() * 16 + bytes.next_u8();
        let j = bytes.next_u8() * 16 + bytes.next_u8();
        let k = bytes.next_u8() * 16 + bytes.next_u8();

        GUID::from_values(a, b, c, [d, e, f, g, h, i, j, k])
    }
}

impl GUID {
    /// Creates a `GUID` represented by the all-zero byte-pattern.
    pub const fn zeroed() -> GUID {
        GUID {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    /// Creates a `GUID` with the given constant values.
    pub const fn from_values(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> GUID {
        GUID {
            data1,
            data2,
            data3,
            data4,
        }
    }

    /// Creates a `GUID` for a "generic" WinRT type.
    pub const fn from_signature(signature: ConstBuffer) -> GUID {
        let data = ConstBuffer::from_slice(&[
            0x11, 0xf4, 0x7a, 0xd5, 0x7b, 0x73, 0x42, 0xc0, 0xab, 0xae, 0x87, 0x8b, 0x1e, 0x16,
            0xad, 0xee,
        ]);

        let data = data.push_other(signature);

        let bytes = const_sha1::sha1(&data).bytes();
        let first = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let second = u16::from_be_bytes([bytes[4], bytes[5]]);
        let mut third = u16::from_be_bytes([bytes[6], bytes[7]]);
        third = (third & 0x0fff) | (5 << 12);
        let fourth = (bytes[8] & 0x3f) | 0x80;

        Self::from_values(
            first,
            second,
            third,
            [
                fourth, bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
            ],
        )
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
