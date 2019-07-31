use super::*;

#[allow(non_upper_case_globals)]
pub const IID_IUnknown: IID = IID {
    data1: 0u32,
    data2: 0u16,
    data3: 0u16,
    data4: [192u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 70u8],
};

#[allow(non_snake_case)]
#[repr(C)]
pub struct IUnknownVTable {
    pub QueryInterface:
        unsafe extern "stdcall" fn(*mut RawIUnknown, *const IID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "stdcall" fn(*mut RawIUnknown) -> u32,
    pub Release: unsafe extern "stdcall" fn(*mut RawIUnknown) -> u32,
}

#[repr(C)]
pub struct RawIUnknown {
    vtable: *const IUnknownVTable,
}

impl RawIUnknown {
    pub unsafe fn raw_query_interface(
        &mut self,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        ((*self.vtable).QueryInterface)(self, riid, ppv)
    }
    pub unsafe fn raw_add_ref(&mut self) -> u32 {
        ((*self.vtable).AddRef)(self)
    }
    pub unsafe fn raw_release(&mut self) -> u32 {
        ((*self.vtable).Release)(self)
    }
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let hr = unsafe { self.raw_query_interface(&T::IID as *const IID, &mut ppv) };
        if failed(hr) {
            assert!(hr == E_NOINTERFACE);
            return None;
        }
        Some(unsafe { ComPtr::new(std::ptr::NonNull::new(ppv as *mut T)?) })
    }
}

#[repr(C)]
pub struct IUnknown {
    inner: RawIUnknown,
}

impl IUnknown {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        self.inner.query_interface()
    }
}

impl ComInterface for IUnknown {
    const IID: IID = IID_IUnknown;
}
