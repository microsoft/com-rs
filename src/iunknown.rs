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

#[macro_export]
macro_rules! iunknown_gen_vtable {
    ($type:ty, $offset:literal) => {{
        unsafe extern "stdcall" fn iunknown_query_interface(
            this: *mut IUnknownVPtr,
            riid: *const IID,
            ppv: *mut *mut c_void,
        ) -> HRESULT {
            let this = this.sub($offset) as *mut $type;
            (*this).query_interface(riid, ppv)
        }
        unsafe extern "stdcall" fn iunknown_add_ref(this: *mut IUnknownVPtr) -> u32 {
            let this = this.sub($offset) as *mut $type;
            (*this).add_ref()
        }
        unsafe extern "stdcall" fn iunknown_release(this: *mut IUnknownVPtr) -> u32 {
            let this = this.sub($offset) as *mut $type;
            (*this).release()
        }

        IUnknownVTable {
            QueryInterface: iunknown_query_interface,
            Release: iunknown_release,
            AddRef: iunknown_add_ref,
        }
    }};
}
