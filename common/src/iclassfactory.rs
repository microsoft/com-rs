use super::*;

// uuid(0x000e0000, 0x0000, 0x0000, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46)
pub const IID_ICLASS_FACTORY: IID = IID {
    data1: 0x01u32,
    data2: 0u16,
    data3: 0u16,
    data4: [0xC0, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0x46u8],
};

#[allow(non_snake_case)]
#[repr(C)]
pub struct IClassFactoryVTable {
    pub CreateInstance:
        unsafe extern "stdcall" fn(*mut RawIUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub LockServer: unsafe extern "stdcall" fn(BOOL) -> HRESULT,
}

#[repr(C)]
pub struct RawIClassFactory {
    iunknown: *const IUnknownVTable,
    vtable: *const IClassFactoryVTable,
}

impl RawIClassFactory {
    pub unsafe fn raw_create_instance(&mut self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
        // TODO: Support aggregation!
        // https://docs.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iclassfactory-createinstance
        ((*self.vtable).CreateInstance)(std::ptr::null_mut(), riid, ppv)
    }

    pub unsafe fn raw_lock_server(&mut self, increment: bool) -> HRESULT {
        ((*self.vtable).LockServer)(increment as BOOL)
    }

    pub fn create_instance<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let mut ppv = std::ptr::null_mut::<c_void>();
        let hr = unsafe { self.raw_create_instance(&T::IID as *const IID, &mut ppv) };
        if failed(hr) {
            // TODO: decide what failures are possible
            return None;
        }
        Some(unsafe { ComPtr::new(std::ptr::NonNull::new(ppv as *mut T)?) })
    }
}

#[repr(C)]
pub struct IClassFactory {
    inner: RawIClassFactory,
}

impl IClassFactory {
    pub fn query_interface<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        let inner: &mut RawIUnknown = self.inner.as_mut();
        inner.query_interface()
    }

    pub fn create_instance<T: ComInterface>(&mut self) -> Option<ComPtr<T>> {
        self.inner.create_instance()
    }
}

impl ComInterface for IClassFactory {
    const IID: IID = IID_IUnknown;
}

impl std::convert::AsRef<RawIUnknown> for RawIClassFactory {
    fn as_ref(&self) -> &RawIUnknown {
        unsafe { &*(self as *const RawIClassFactory as *const RawIUnknown) }
    }
}

impl std::convert::AsMut<RawIUnknown> for RawIClassFactory {
    fn as_mut(&mut self) -> &mut RawIUnknown {
        unsafe { &mut *(self as *mut RawIClassFactory as *mut RawIUnknown) }
    }
}
