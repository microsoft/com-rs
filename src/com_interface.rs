use crate::interfaces::IUnknown;
use crate::sys::IID;

/// A COM compliant interface pointer
///
/// # Safety
///
/// The struct implementing this trait must provide a valid vtable as the
/// associated VTable type. A vtable is valid if:
/// * it is `#[repr(C)]`
/// * the type only contains `extern "stdcall" fn" definitions
///
/// The implementor must be a transparrently equivalent to a valid interface pointer
/// for the interface `T`. An interface pointer as the name suggests points to an
/// interface. A valid interface is itself trivial castable to a `*mut T::VTable`.
/// In other words, the implementing type must also be equal to `*mut *const T::VTable`
pub unsafe trait ComInterface: Sized + 'static {
    /// A COM compatible V-Table
    type VTable;
    /// The interface that this interface inherits from
    type Super: ComInterface;
    /// The associated id for this interface
    const IID: IID;

    /// Check whether a given IID is in the inheritance hierarchy of this interface
    fn is_iid_in_inheritance_chain(riid: &IID) -> bool {
        riid == &Self::IID
            || (Self::IID != <IUnknown as ComInterface>::IID
                && <Self::Super as ComInterface>::is_iid_in_inheritance_chain(riid))
    }

    /// Cast the interface pointer to a pointer to IUnknown.
    fn as_iunknown(&self) -> &IUnknown {
        unsafe { std::mem::transmute(self) }
    }

    /// Cast the COM interface pointer to a raw pointer
    ///
    /// The returned pointer is only guranteed valid for as long
    /// as the reference to self id valid.
    fn as_raw(&self) -> std::ptr::NonNull<std::ptr::NonNull<Self::VTable>> {
        unsafe { std::mem::transmute_copy(self) }
    }
}
