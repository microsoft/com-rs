/// A COM compliant class
///
/// # Safety
///
/// The implementing struct must have the following properties:
/// * it is `#[repr(C)]`
/// * The first fields of the struct are pointers to the backing VTables for
/// each of the COM Interfaces the class implements
pub unsafe trait Class {
    /// The factory object associated with this class
    type Factory;

    /// Decrement the current reference count and return the new count
    fn dec_ref_count(&self) -> u32;
}

/// An allocated COM class
///
/// The class must be heap allocated and not be moved in memory.
/// This wrapper decrements the inner class ref count when dropped
/// and frees the heap allocation as well as the class itself when
/// that ref count is 0.
#[repr(transparent)]
pub struct ClassAllocation<T: Class> {
    inner: std::mem::ManuallyDrop<std::pin::Pin<Box<T>>>,
}

impl<T: Class> ClassAllocation<T> {
    /// Create a new class allocation
    ///
    /// This is not normally used by users of the COM crate but by the code generator
    pub fn new(inner: std::pin::Pin<Box<T>>) -> Self {
        Self {
            inner: std::mem::ManuallyDrop::new(inner),
        }
    }

    /// Create an allocated class from a raw pointer
    ///
    /// # Safety
    /// Must be a valid, owned pointer to an allocated COM class. This returns an owned `ClassAllocation`
    /// which will drop the wrapped COM class when it is dropped.
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        let inner = std::mem::ManuallyDrop::new(Box::from_raw(raw).into());
        Self { inner }
    }
}

impl<T: Class> std::ops::Deref for ClassAllocation<T> {
    type Target = std::pin::Pin<Box<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Class> Drop for ClassAllocation<T> {
    fn drop(&mut self) {
        if self.inner.dec_ref_count() == 0 {
            // SAFETY: This is safe because the inner value is not accessible by anyone else
            unsafe {
                std::mem::ManuallyDrop::drop(&mut self.inner);
            }
        }
    }
}

impl<T: Class> std::fmt::Debug for ClassAllocation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = self.inner.as_ref().get_ref() as *const T;
        f.debug_struct("ClassAllocation")
            .field("ptr", &ptr)
            .finish()
    }
}
