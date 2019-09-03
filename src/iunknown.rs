use com_interface_attribute::com_interface;
use winapi::{
    ctypes::c_void,
    shared::{guiddef::REFIID, ntdef::HRESULT},
};

#[com_interface(00000000-0000-0000-C000-000000000046)]
pub trait IUnknown {
    fn query_interface(&mut self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    fn add_ref(&mut self) -> u32;
    fn release(&mut self) -> u32;
}
