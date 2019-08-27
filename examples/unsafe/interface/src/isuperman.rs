use winapi::shared::guiddef::IID;
use com::{ComInterface, ComPtr, IUnknown, IUnknownMethods};
use winapi::um::winnt::HRESULT;

pub const IID_ISUPERMAN: IID = IID {
    Data1: 0xa56b76e4,
    Data2: 0x4bd7,
    Data3: 0x48b9,
    Data4: [0x8a, 0xf6, 0xb9, 0x3f, 0x43, 0xe8, 0x69, 0xc8],
};

pub trait ISuperman: IUnknown {
    // [in]
    fn take_input(&mut self, in_var: u32) -> HRESULT;

    // [out]
    fn populate_output(&mut self, out_var: *mut u32) -> HRESULT;

    // [in, out]
    fn mutate_and_return(&mut self, in_out_var: *mut u32) -> HRESULT;

    // [in] pointer
    fn take_input_ptr(&mut self, in_ptr_var: *const u32) -> HRESULT;

    // // [in, out] Interface
    // fn take_interface();

    // // [out] Interface
    // fn populate_interface(ComOutPtr<ComItf>);
    
}

unsafe impl ComInterface for ISuperman {
    const IID: IID = IID_ISUPERMAN;
}

pub type ISupermanVPtr = *const ISupermanVTable;

impl <T: ISuperman + ComInterface + ?Sized> ISuperman for ComPtr<T> {
    fn take_input(&mut self, in_var: u32) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ISupermanVPtr;
        unsafe { ((**itf_ptr).1.TakeInput)(itf_ptr, in_var) }
    }

    fn populate_output(&mut self, out_var: *mut u32) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ISupermanVPtr;
        unsafe { ((**itf_ptr).1.PopulateOutput)(itf_ptr, out_var) }
    }

    fn mutate_and_return(&mut self, in_out_var: *mut u32) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ISupermanVPtr;
        unsafe { ((**itf_ptr).1.MutateAndReturn)(itf_ptr, in_out_var) }
    }

    fn take_input_ptr(&mut self, in_ptr_var: *const u32) -> HRESULT {
        let itf_ptr = self.into_raw() as *mut ISupermanVPtr;
        unsafe { ((**itf_ptr).1.TakeInputPtr)(itf_ptr, in_ptr_var) }
    }

}

#[repr(C)]
pub struct ISupermanVTable(pub IUnknownMethods, pub ISupermanMethods);

#[allow(non_snake_case)]
#[repr(C)]
pub struct ISupermanMethods {
    pub TakeInput: unsafe extern "stdcall" fn(*mut ISupermanVPtr, in_var: u32) -> HRESULT,
    pub PopulateOutput: unsafe extern "stdcall" fn(*mut ISupermanVPtr, out_var: *mut u32) -> HRESULT,
    pub MutateAndReturn: unsafe extern "stdcall" fn(*mut ISupermanVPtr, in_out_var: *mut u32) -> HRESULT,
    pub TakeInputPtr: unsafe extern "stdcall" fn(*mut ISupermanVPtr, in_ptr_var: *const u32) -> HRESULT,
}
