#![allow(clippy::missing_safety_doc)]

use alloc::boxed::Box;
use core::mem::ManuallyDrop;

#[cfg(doc)]
use crate::interfaces::IUnknown;

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
    ///
    /// # Safety
    ///
    /// Because the caller is directly modifying the reference count of an
    /// object, and reference counts are used to determine object lifetime,
    /// the caller is responsible for ensuring that the object is destroyed
    /// if `dec_ref_count` reaches zero. All such adjustments to the
    /// reference count can only be used by `unsafe` code, because this method
    /// has a side effect (modifies the reference count) but this side effect
    /// is not represented in Rust's type system (no refcount-holding object
    /// is destroyed).
    ///
    /// This method should only be called in [`Drop`] implementations, or similar
    /// functions that terminate the lifetime of a reference-holding type.
    unsafe fn dec_ref_count(&self) -> u32;

    /// Increment the current reference count and return the new count
    ///
    /// # Safety
    ///
    /// Because the caller is directly modifying the reference count of an
    /// object, and reference counts are used to determine object lifetime,
    /// the caller is responsible for ensuring that the newly-created reference
    /// is correctly encapsulated within a Rust object. All such adjustments to
    /// the reference count can only be used by `unsafe` code, because this
    /// method has a side effect (modifies the reference count) but this side
    /// effect is not represented in Rust's type system (no refcount-holding
    /// object is destroyed).
    ///
    /// This method should only be called in type constructors, [`Clone`]
    /// implementations, [`IUnknown::query_interface()`] implementations, or similar
    /// code paths that create a new instance of a Rust type that holds the
    /// counted reference.
    unsafe fn add_ref(&self) -> u32;
}

/// An allocated COM class
///
/// The class must be heap allocated and not be moved in memory.
/// This wrapper decrements the inner class ref count when dropped
/// and frees the heap allocation as well as the class itself when
/// that ref count is 0.
#[repr(transparent)]
pub struct ClassAllocation<T: Class> {
    inner: core::mem::ManuallyDrop<core::pin::Pin<Box<T>>>,
}

impl<T: Class> ClassAllocation<T> {
    /// Create a new class allocation
    ///
    /// This is not normally used by users of the COM crate but by the code generator
    pub fn new(inner: core::pin::Pin<Box<T>>) -> Self {
        Self {
            inner: core::mem::ManuallyDrop::new(inner),
        }
    }

    /// Create an allocated class from a raw pointer
    ///
    /// # Safety
    /// Must be a valid, owned pointer to an allocated COM class. This returns an owned [`ClassAllocation`]
    /// which will drop the wrapped COM class when it is dropped.
    #[inline(always)]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        let inner = core::mem::ManuallyDrop::new(Box::from_raw(raw).into());
        Self { inner }
    }

    /// Drop (free) the inner allocation.
    ///
    /// This function is never inlined, so that the (relatively) cold path of
    /// freeing an object is kept out of the inlined `Release` call paths.
    #[doc(hidden)]
    #[inline(never)]
    pub unsafe fn drop_inner(&mut self) {
        ManuallyDrop::drop(&mut self.inner);
    }
}

/// [`ClassAllocation<T>`] is [`Send`] because it represents an owned reference to
/// a heap allocation, and the changes to that reference count are atomic.
unsafe impl<T: Class> Send for ClassAllocation<T> {}

/// [`ClassAllocation<T>`] is [`Sync`] because it represents an aliased (shared)
/// reference to a heap-allocated object, and the only way you can gain access
/// to that heap object is to acquire a `&self` (shared) reference.
unsafe impl<T: Class> Sync for ClassAllocation<T> {}

impl<T: Class> core::ops::Deref for ClassAllocation<T> {
    type Target = core::pin::Pin<Box<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Class> Drop for ClassAllocation<T> {
    fn drop(&mut self) {
        unsafe {
            if self.inner.dec_ref_count() == 0 {
                // SAFETY: This is safe because the inner value is not accessible by anyone else
                core::mem::ManuallyDrop::drop(&mut self.inner);
            }
        }
    }
}

impl<T: Class + core::fmt::Debug> core::fmt::Debug for ClassAllocation<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let inner = self.inner.as_ref().get_ref();
        write!(f, "{:?}", inner)
    }
}

impl<T: Class> Clone for ClassAllocation<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.inner.add_ref();
            core::mem::transmute_copy(self)
        }
    }
}
