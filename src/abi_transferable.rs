/// Types that are safe to transfer over a COM API boundary.
///
/// # Safety
/// Implementing types only have associated `Abi` types that are
/// safe to transfer over a COM FFI boundary. Implementing types
/// must also be exactly equivalent to their associated types
/// in layout and abi such that it is safe to transmute between the
/// two types.
pub unsafe trait AbiTransferable: Sized {
    /// The FFI compatible type the implementing type can turn into.
    type Abi;

    /// Turn the type into the FFI ABI type.
    fn get_abi(&self) -> Self::Abi;
    /// Set the abi of the implementing type
    fn set_abi(&mut self) -> *mut Self::Abi;

    /// Convert into a reference to Self from a reference to the ABI
    fn from_abi(abi: Self::Abi) -> Self {
        // This must be safe for the implementing type to
        // correctly implement this trait.
        unsafe { core::mem::transmute_copy(&abi) }
    }

    /// Convert a pointer to a `Self::Abi` and and a length to a slice.
    ///
    /// # Safety
    /// The `abi` pointer must be a valid pointer to an array of `Self::Abi` items of
    /// `len` size for the lifetime `'a`. Nothing can mutate that array while
    /// the slice exists.
    unsafe fn slice_from_abi<'a>(abi: *const Self::Abi, len: usize) -> &'a [Self] {
        core::slice::from_raw_parts(core::mem::transmute_copy(&abi), len)
    }

    /// Convert a pointer to a `Self::Abi` and and a length to a mutable slice.
    ///
    /// # Safety
    /// The same rules apply as with `slice_from_abi` but no other references into
    /// the slice are allowed while the slice exists.
    unsafe fn slice_from_mut_abi<'a>(abi: *mut Self::Abi, len: usize) -> &'a mut [Self] {
        core::slice::from_raw_parts_mut(core::mem::transmute_copy(&abi), len)
    }

    /// Converts and consumes the ABI transferable type into its ABI representation.
    fn into_abi(self) -> Self::Abi {
        // This must be safe for the implementing type to
        // correctly implement this trait.
        let abi = unsafe { core::mem::transmute_copy(&self) };
        core::mem::forget(self);
        abi
    }
}

macro_rules! primitive_transferable_type {
    ($($t:ty),+) => {
        $(unsafe impl AbiTransferable for $t {
            type Abi = Self;
            fn get_abi(&self) -> Self::Abi {
                *self
            }
            fn set_abi(&mut self) -> *mut Self::Abi {
                self as *mut Self::Abi
            }
        })*
    };
}

primitive_transferable_type! {
    bool,
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    f32,
    f64,
    usize,
    crate::sys::GUID
}

unsafe impl<T> AbiTransferable for *mut T {
    type Abi = Self;
    fn get_abi(&self) -> Self::Abi {
        *self
    }
    fn set_abi(&mut self) -> *mut Self::Abi {
        self as *mut Self::Abi
    }
}

unsafe impl<T> AbiTransferable for *const T {
    type Abi = Self;
    fn get_abi(&self) -> Self::Abi {
        *self
    }
    fn set_abi(&mut self) -> *mut Self::Abi {
        self as *mut Self::Abi
    }
}

unsafe impl<T: crate::Interface> AbiTransferable for T {
    type Abi = core::ptr::NonNull<core::ptr::NonNull<<T as crate::Interface>::VTable>>;
    fn get_abi(&self) -> Self::Abi {
        self.as_raw()
    }

    fn set_abi(&mut self) -> *mut Self::Abi {
        &mut self.as_raw()
    }
}

unsafe impl<T: crate::Interface> AbiTransferable for Option<T> {
    type Abi = *mut core::ptr::NonNull<<T as crate::Interface>::VTable>;
    fn get_abi(&self) -> Self::Abi {
        self.as_ref()
            .map(|p| p.as_raw().as_ptr())
            .unwrap_or(::core::ptr::null_mut())
    }

    fn set_abi(&mut self) -> *mut Self::Abi {
        &mut self
            .as_mut()
            .map(|p| p.as_raw().as_ptr())
            .unwrap_or(::core::ptr::null_mut())
    }
}
