use super::*;
use com_interface_attribute::com_interface;
use winapi::ctypes::c_void;
use winapi::shared::guiddef::GUID;
use winapi::shared::ntdef::HRESULT;

#[com_interface(00000000-0000-0000-C000-000000000046)]
pub trait IUnknown {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT;
    fn add_ref(&mut self) -> u32;
    fn release(&mut self) -> u32;
}

// --------------- DESIRED EXPANSION --------------
// use super::*;
// use winapi::shared::guiddef::GUID;
// use winapi::shared::ntdef::HRESULT;
// use winapi::ctypes::c_void;

// #[allow(non_upper_case_globals)]
// pub const IID_IUNKNOWN: GUID = GUID {
//     Data1: 0u32,
//     Data2: 0u16,
//     Data3: 0u16,
//     Data4: [192u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 70u8],
// };

// #[allow(non_snake_case)]
// #[repr(C)]
// pub struct IUnknownVTable {
//     pub QueryInterface:
//         unsafe extern "stdcall" fn(*mut IUnknownVPtr, *const IID, *mut *mut c_void) -> HRESULT,
//     pub AddRef: unsafe extern "stdcall" fn(*mut IUnknownVPtr) -> u32,
//     pub Release: unsafe extern "stdcall" fn(*mut IUnknownVPtr) -> u32,
// }

// pub type IUnknownVPtr = *const IUnknownVTable;

// pub trait IUnknown {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT;
//     fn add_ref(&mut self) -> u32;
//     fn release(&mut self) -> u32;
// }

// impl <T: IUnknown + ComInterface + ?Sized> IUnknown for ComPtr<T> {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).QueryInterface)(itf_ptr, riid, ppv) }
//     }

//     fn add_ref(&mut self) -> u32 {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).AddRef)(itf_ptr) }
//     }

//     fn release(&mut self) -> u32 {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).Release)(itf_ptr) }
//     }
// }

// unsafe impl ComInterface for IUnknown {
//     type VTable = IUnknownVTable;
//     const IID: IID = IID_IUNKNOWN;
// }

// impl<T: IUnknown + ComInterface + ?Sized> ComPtr<T> {
//     fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).QueryInterface)(itf_ptr, riid, ppv) }
//     }

//     fn add_ref(&mut self) -> u32 {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).AddRef)(itf_ptr) }
//     }

//     fn release(&mut self) -> u32 {
//         let itf_ptr = self.into_raw() as *mut IUnknownVPtr;
//         unsafe { ((**itf_ptr).Release)(itf_ptr) }
//     }
// }
