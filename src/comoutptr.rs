use std::mem::MaybeUninit;

/// Represent the possible modes of ComOut
enum ComOutValue<T> {
    Owned(MaybeUninit<T>),
    Pointer(*const T),
}

/// Represent the possible states of ComOut
enum ComValueState {
    Uninitialized,
    Initialized,
}

/// This marker indicates that the structs are sized and don't have pointer to other data
pub unsafe trait ComTrivialStruct: Sized {}

// XXX_ couldn't make this macro work..
// Implement ComTrivialStruct for some primitive types
// macro_rules! impl_comtrivial_struct {
//     ($($x:ty),+) => {
//         $(unsafe impl ComTrivialStruct for $x {}),+
//     }
// }
// impl_comtrivial_struct!(u8, i8, u16, i16, u32, i32, usize, isize);
unsafe impl ComTrivialStruct for u8 {}
unsafe impl ComTrivialStruct for i8 {}
unsafe impl ComTrivialStruct for u16 {}
unsafe impl ComTrivialStruct for i16 {}
unsafe impl ComTrivialStruct for u32 {}
unsafe impl ComTrivialStruct for i32 {}
unsafe impl ComTrivialStruct for usize {}
unsafe impl ComTrivialStruct for isize {}

// Wrapper struct for a potentially uninitialised raw pointer.
// Simulates uninitialisation using a boolean.
pub struct ComOut<T> {
    value: ComOutValue<T>,
    state: ComValueState,
}

impl<T> ComOut<T> {
    /// Used by the caller to allocate the memory passed to the callee
    pub fn new_uninit() -> Self {
        ComOut {
            value: ComOutValue::Owned(MaybeUninit::<T>::uninit()),
            state: ComValueState::Uninitialized,
        }
    }

    /// Used by the callee to reference the memory allocated by the caller
    /// Important! We are going to always ignore the value pointed at by ptr here.
    /// The only way you can "get" a value is by first setting it, so we know it's
    /// initialised.
    ///
    /// I think this fits nicely with the [out] parameter assumption by MSDN, that
    /// We can treat all [out] parameters as undefined as the server. Hence, all
    /// creation methods must assume uninitialised state.
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        ComOut {
            value: ComOutValue::Pointer(ptr),
            state: ComValueState::Uninitialized,
        }
    }

    /// Used by the caller for optional arguments that shouldn't be set
    pub fn new_null() -> Self {
        ComOut {
            value: ComOutValue::Pointer(std::ptr::null()),
            state: ComValueState::Uninitialized,
        }
    }

    unsafe fn _unwrap(self) -> Option<T> {
        match self.state {
            ComValueState::Uninitialized => None,
            ComValueState::Initialized => match self.value {
                ComOutValue::Owned(value) => Some(value.assume_init()),
                ComOutValue::Pointer(_) => None,
            },
        }
    }

    /// Marks the internal data as initialized
    /// It's unsafe to mark internal data as initialized when it wasn't because it allows
    /// to trigger undefined behaviour with the functions `unwrap` and `get`
    pub unsafe fn mark_init(&mut self) {
        self.state = ComValueState::Initialized;
    }

    /// Unwraps the internal data.
    /// Only works if the data is owned by the wrapper (doesn't work if it owns a pointer).
    /// Only returns the unwrapped value if the data is marked as initialized.
    #[cfg(feature = "comout_trust_callee")]
    pub fn unwrap(self) -> Option<T> {
        unsafe { self._unwrap() }
    }

    /// Unwraps the internal data.
    /// Only works if the data is owned by the wrapper (doesn't work if it owns a pointer).
    /// Only returns the unwrapped value if the data is marked as initialized.
    #[cfg(not(feature = "comout_trust_callee"))]
    pub unsafe fn unwrap(self) -> Option<T> {
        self._unwrap()
    }

    /// Write the pointer with the new value
    unsafe fn _write_ptr(&mut self, value: T) {
        std::ptr::write(self.as_mut_ptr(), value);
    }

    /// Used by the callee to copy the output into the referenced memory.
    /// The callee must always check if the pointer is not null by using
    /// `is_null()`.
    /// If the internal pointer is null, it will panic instead of triggering a null pointer dereference.
    pub fn set(&mut self, value: T)
    where
        T: ComTrivialStruct,
    {
        // Panic if it's null, in C/C++ this would also crash
        if let ComOutValue::Pointer(value) = self.value {
            if value == std::ptr::null() {
                panic!("Trying to write to a null pointer");
            }
        }
        match self.state {
            ComValueState::Uninitialized => unsafe { self._write_ptr(value) },
            // Drop the previous one
            ComValueState::Initialized => unsafe {
                std::ptr::drop_in_place(self.as_mut_ptr());
                self._write_ptr(value);
            },
        };
    }

    unsafe fn _get(&self) -> Option<&T> {
        match self.state {
            ComValueState::Uninitialized => None,
            ComValueState::Initialized => Some(&*self.as_ptr()),
        }
    }

    /// Gets a reference to the internal data.
    /// Only returns the reference if the data is marked as initialized.
    #[cfg(feature = "comout_trust_callee")]
    pub fn get(&self) -> Option<&T> {
        unsafe { self._get() }
    }

    /// Gets a reference to the internal data.
    /// Only returns the reference if the data is marked as initialized.
    #[cfg(not(feature = "comout_trust_callee"))]
    pub unsafe fn get(&self) -> Option<&T> {
        self._get()
    }

    /// Primarily used for FFI to get the internal pointer.
    /// Reading from it if it hasn't been initialized is undefined behaviour.
    pub fn as_ptr(&self) -> *const T {
        match &self.value {
            ComOutValue::Owned(value) => value.as_ptr(),
            ComOutValue::Pointer(value) => *value,
        }
    }

    /// Primarily used for FFI to get them to write to the underlying
    /// pointer address.
    /// Reading from it if it hasn't been initialized is undefined behaviour.
    /// The user must check if the pointer is not null.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        match &mut self.value {
            ComOutValue::Owned(value) => value.as_mut_ptr(),
            ComOutValue::Pointer(value) => *value as *mut T,
        }
    }

    /// Check if pointer is null
    pub fn is_null(&self) -> bool {
        match self.value {
            ComOutValue::Owned(_) => true,
            ComOutValue::Pointer(value) => value == std::ptr::null(),
        }
    }
}

#[cfg(test)]
mod test_com_out {
    use super::ComOut;

    type HRESULT = i32;

    fn wrapped_callee(num: &mut ComOut<u32>) -> HRESULT {
        num.set(10);
        0
    }

    fn unsafe_callee(num: *mut u32) -> HRESULT {
        wrapped_callee(&mut unsafe { ComOut::from_ptr(num) });
        0
    }

    fn generated_caller(out: &mut ComOut<u32>) -> Result<HRESULT, ()> {
        let res = unsafe_callee(out.as_mut_ptr());
        if res >= 0 {
            unsafe {
                out.mark_init();
            }
        }
        Ok(res)
    }

    #[test]
    fn safe_caller() {
        let mut out: ComOut<u32> = ComOut::new_uninit();
        let res = generated_caller(&mut out);
        let val = match res {
            Ok(_) => unsafe { out.unwrap() },
            Err(_) => None,
        };
        assert_eq!(val, Some(10));
    }

}
