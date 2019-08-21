use crate::clark_kent::{ClarkKent};
use com::{
    IClassFactoryMethods, IUnknownMethods, IID_ICLASS_FACTORY,
    IID_IUNKNOWN, IClassFactory, IUnknownVPtr, IClassFactoryVPtr, IUnknown, IClassFactoryVTable
};

use winapi::{
    ctypes::c_void,
    shared::{
        guiddef::{IsEqualGUID, IID, REFIID},
        minwindef::BOOL,
        winerror::{CLASS_E_NOAGGREGATION, E_NOINTERFACE, HRESULT, NOERROR, S_OK},
    },
};

#[repr(C)]
pub struct ClarkKentClass {
    inner: IClassFactoryVPtr,
    ref_count: u32,
}

impl IClassFactory for ClarkKentClass {
    fn create_instance(&mut self, aggr: *mut IUnknownVPtr, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
        println!("Creating instance...");
        if !aggr.is_null() {
            return CLASS_E_NOAGGREGATION;
        }

        let mut ck = Box::new(ClarkKent::new());
        ck.add_ref();
        let hr = ck.query_interface(riid, ppv);
        ck.release();
        let ptr = Box::into_raw(ck);

        hr
    }

    fn lock_server(&mut self, increment: BOOL) -> HRESULT {
        println!("LockServer called");
        S_OK
    }
}

impl IUnknown for ClarkKentClass {
    fn query_interface(&mut self, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT {
        /* TODO: This should be the safe wrapper. You shouldn't need to write unsafe code here. */
        unsafe {
            let riid = &*riid;
            if IsEqualGUID(riid, &IID_IUNKNOWN) || IsEqualGUID(riid, &IID_ICLASS_FACTORY)
            {
                *ppv = self as *const _ as *mut c_void;
                self.add_ref();
                NOERROR
            } else {
                E_NOINTERFACE
            }
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
            println!("Count is 0 for ClarkKentClass. Freeing memory...");
            drop(self);
        }
        count
    }
}

impl Drop for ClarkKentClass {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.inner as *mut IClassFactoryVTable) };
    }
}

unsafe extern "stdcall" fn query_interface(
    this: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    println!("Querying interface on ClarkKentClass...");
    let this = this as *mut ClarkKentClass;
    (*this).query_interface(riid, ppv)
}

unsafe extern "stdcall" fn add_ref(this: *mut IUnknownVPtr) -> u32 {
    println!("Adding ref...");
    let this = this as *mut ClarkKentClass;
    (*this).add_ref()
}

// TODO: This could potentially be null or pointing to some invalid memory
unsafe extern "stdcall" fn release(this: *mut IUnknownVPtr) -> u32 {
    println!("Releasing...");
    let this = this as *mut ClarkKentClass;
    (*this).release()
}

unsafe extern "stdcall" fn create_instance(
    this: *mut IClassFactoryVPtr,
    aggregate: *mut IUnknownVPtr,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let this = this as *mut ClarkKentClass;
    (*this).create_instance(aggregate, riid, ppv)
}

unsafe extern "stdcall" fn lock_server(this: *mut IClassFactoryVPtr, increment: BOOL) -> HRESULT {
    let this = this as *mut ClarkKentClass;
    (*this).lock_server(increment)
}

impl ClarkKentClass {
    pub(crate) fn new() -> ClarkKentClass {
        println!("Allocating new Vtable for ClarkKentClass...");
        let iunknown = IUnknownMethods {
            QueryInterface: query_interface,
            Release: release,
            AddRef: add_ref,
        };
        let iclassfactory = IClassFactoryMethods {
            CreateInstance: create_instance,
            LockServer: lock_server,
        };
        let vptr = Box::into_raw(Box::new(IClassFactoryVTable(
            iunknown,
            iclassfactory,
        )));
        ClarkKentClass {
            inner: vptr,
            ref_count: 0,
        }
    }
}
