// Wrapper struct for a potentially uninitialised raw pointer.
// Simulates uninitialisation using a boolean.
pub struct ComOutPtr<T> {
    value: *const T,
    init: bool
}

impl<T> ComOutPtr<T> {
    pub fn new() -> Self {
        ComOutPtr {
            value: std::ptr::null(),
            init: false
        }
    }    

    pub fn get(&self) -> Option<&T> {
        if self.init && !self.value.is_null() {
            Some(unsafe {&*self.value})
        } else {
            // Uninitialised or null.
            None
        }
    }

    pub fn set(&mut self, val: T) {
        if self.value.is_null() {
            // Doesn't point to valid address. In that case, we replace
            // the null pointer with a new one.
            self.value = Box::into_raw(Box::new(val));
        } else {
            unsafe { std::ptr::write(self.value as *mut T, val); }
        }

        self.init = true;
    }

    // Primarily used for FFI to get them to write to the underlying
    // pointer address. However, not feasible with our null pointer way.
    // pub fn as_mut_ptr(&self) -> *mut T {
    //     self.value as *mut T
    // }

    // Important! We are going to always ignore the value pointed at by ptr here.
    // The only way you can "get" a value is by first setting it, so we know it's
    // initialised.
    //
    // I think this fits nicely with the [out] parameter assumption by MSDN, that
    // We can treat all [out] parameters as undefined as the server. Hence, all
    // creation methods must assume uninitialised state.
    pub fn from_ptr(ptr: *mut T) -> Self {
        ComOutPtr {
            value: ptr,
            init: false
        }
    }

    // What do we do with old value here? Do we drop it if initialised or what?
    // This is definitely unsafe, as value could be an uninitialised FFI pointer.
    pub unsafe fn wrap(&mut self, value: *mut T) {
        self.value = value;
        self.init = true;
    }
}