use com_interface_attribute::com_interface;

#[com_interface(00000000-0000-0000-C000-000000000046)]
pub trait IUnknown {
    fn query_interface(
        &mut self, 
        riid: com::_winapi::shared::guiddef::REFIID, 
        ppv: *mut *mut com::_winapi::ctypes::c_void
    ) -> com::_winapi::shared::ntdef::HRESULT;
    fn add_ref(&mut self) -> u32;
    fn release(&mut self) -> u32;
}
