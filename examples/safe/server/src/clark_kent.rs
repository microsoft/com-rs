use com::{ComOut, IUnknown, IUnknownVPtr, IUnknownVTable, IID_IUNKNOWN};
use interface::isuperman::{ISuperman, ISupermanVPtr, ISupermanVTable, IID_ISUPERMAN};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID},
        winerror::{E_FAIL, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
    },
};

#[repr(C)]
pub struct ClarkKent {
    inner: ISupermanVPtr,
    ref_count: u32,
}

impl Drop for ClarkKent {
    fn drop(&mut self) {
        let _ = unsafe {
            Box::from_raw(self.inner as *mut ISupermanVTable);
        };
    }
}

impl ISuperman for ClarkKent {
    fn take_input(&mut self, in_var: u32) -> HRESULT {
        println!("Received Input! Input is: {}", in_var);
        if in_var > 5 {
            return E_FAIL;
        }

        S_OK
    }

    fn populate_output(&mut self, out_var: &mut ComOut<u32>) -> HRESULT {
        out_var.set(6);
        S_OK
    }

    fn mutate_and_return(&mut self, in_out_var: &mut Option<Box<u32>>) -> HRESULT {
        match in_out_var {
            Some(n) => **n = 100,
            None => println!("Received null, unable to mutate!"),
        };

        S_OK
    }

    fn take_input_ptr(&mut self, in_ptr_var: &Option<u32>) -> HRESULT {
        match in_ptr_var {
            Some(n) => {
                if *n > 5 {
                    return E_FAIL;
                } else {
                    return S_OK;
                }
            }
            None => {
                return E_FAIL;
            }
        };
    }
}

impl IUnknown for ClarkKent {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;

            if IsEqualGUID(riid, &IID_IUNKNOWN) || IsEqualGUID(riid, &IID_ISUPERMAN) {
                *ppv = &self.inner as *const _ as *mut c_void;
            } else {
                println!("Returning NO INTERFACE.");
                return E_NOINTERFACE;
            }

            println!("Successful!.");
            self.add_ref();
            NOERROR
        }
    }

    fn add_ref(&mut self) -> u32 {
        self.ref_count += 1;
        println!("Count now {}", self.ref_count);
        self.ref_count
    }

    fn release(&mut self) -> u32 {
        self.ref_count -= 1;
        println!("Count now {}", self.ref_count);
        let count = self.ref_count;
        if count == 0 {
            println!("Count is 0 for ClarkKent. Freeing memory...");
            drop(self)
        }
        count
    }
}

// Adjustor Thunks for ISuperman
unsafe extern "stdcall" fn query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = this as *mut ClarkKent;
    (*this).query_interface(riid, ppv)
}

unsafe extern "stdcall" fn add_ref(this: *mut IUnknownVPtr) -> u32 {
    println!("Adding ref...");
    let this = this as *mut ClarkKent;
    (*this).add_ref()
}

unsafe extern "stdcall" fn release(this: *mut IUnknownVPtr) -> u32 {
    println!("Releasing...");
    let this = this as *mut ClarkKent;
    (*this).release()
}

unsafe extern "stdcall" fn take_input(this: *mut ISupermanVPtr, in_var: u32) -> HRESULT {
    let this = this as *mut ClarkKent;
    (*this).take_input(in_var)
}

unsafe extern "stdcall" fn populate_output(this: *mut ISupermanVPtr, out_var: *mut u32) -> HRESULT {
    let this = this as *mut ClarkKent;
    let mut ptr = ComOut::from_ptr(out_var);
    (*this).populate_output(&mut ptr)
}

unsafe extern "stdcall" fn mutate_and_return(
    this: *mut ISupermanVPtr,
    in_out_var: *mut u32,
) -> HRESULT {
    let this = this as *mut ClarkKent;
    let mut opt = if in_out_var.is_null() {
        None
    } else {
        Some(Box::from_raw(in_out_var))
    };

    let hr = (*this).mutate_and_return(&mut opt);

    // Server side should not be responsible for memory allocated by client.
    match opt {
        Some(n) => {
            Box::into_raw(n);
        }
        _ => (),
    };

    hr
}

unsafe extern "stdcall" fn take_input_ptr(
    this: *mut ISupermanVPtr,
    in_ptr_var: *const u32,
) -> HRESULT {
    let this = this as *mut ClarkKent;
    let opt = if in_ptr_var.is_null() {
        None
    } else {
        Some(*in_ptr_var)
    };

    (*this).take_input_ptr(&opt)
}

impl ClarkKent {
    pub(crate) fn new() -> ClarkKent {
        println!("Allocating new Vtable...");

        /* Initialising VTable for ISuperman */
        let iunknown = IUnknownVTable {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let isuperman = ISupermanVTable {
            base: iunknown,
            TakeInput: take_input,
            PopulateOutput: populate_output,
            MutateAndReturn: mutate_and_return,
            TakeInputPtr: take_input_ptr,
        };
        let vptr = Box::into_raw(Box::new(isuperman));

        ClarkKent {
            inner: vptr,
            ref_count: 0,
        }
    }
}
