use crate::AbiTransferable;

/// A COM method parameter used to accept either a reference or value.
pub enum Param<'a, T> {
    /// The borrowed version of the param
    Borrowed(&'a T),
    /// The owned version of the param
    Owned(T),
}

impl<'a, T: AbiTransferable> Param<'a, T> {
    /// Get the param's underlying ABI
    pub fn get_abi(&mut self) -> T::Abi {
        match self {
            Param::Borrowed(value) => value.get_abi(),
            Param::Owned(value) => value.get_abi(),
        }
    }
}

impl<'a, T> From<T> for Param<'a, T> {
    fn from(value: T) -> Param<'a, T> {
        Param::Owned(value)
    }
}

impl<'a, T> From<&'a T> for Param<'a, T> {
    fn from(value: &'a T) -> Param<'a, T> {
        Param::Borrowed(value)
    }
}

impl<'a, T> From<&'a T> for Param<'a, *const T> {
    fn from(value: &'a T) -> Param<'a, *const T> {
        Param::Owned(value)
    }
}

impl<'a, T> From<*mut T> for Param<'a, *const T> {
    fn from(value: *mut T) -> Param<'a, *const T> {
        Param::Owned(value)
    }
}

impl<'a, T> From<&'a mut T> for Param<'a, *mut T> {
    fn from(value: &'a mut T) -> Param<'a, *mut T> {
        Param::Owned(value)
    }
}
