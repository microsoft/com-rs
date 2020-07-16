use super::offset;
use crate::ComInterface;

/// A COM compliant class
///
/// # Safety
///
/// The implementing struct must have the following properties:
/// * it is `#[repr(C)]`
/// * The first fields of the struct are pointers to the backing VTables for
/// each of the COM Interfaces the class implements
pub unsafe trait CoClass {}

/// A COM interface that will be exposed in a COM server
pub trait ProductionComInterface<T>: ComInterface {
    /// Get the vtable for a particular COM interface
    fn vtable<O: offset::Offset>() -> Self::VTable;
}

#[doc(hidden)]
#[macro_export]
macro_rules! vtable {
    ($class:ident: $interface:ident, $offset:ident) => {
        <$interface as $crate::production::ProductionComInterface<$class>>::vtable::<
            $crate::production::offset::$offset,
        >();
    };
    ($class:ident: $interface:ident, 4usize) => {
        $crate::vtable!($class: $interface, Four)
    };
    ($class:ident: $interface:ident, 3usize) => {
        $crate::vtable!($class: $interface, Three)
    };
    ($class:ident: $interface:ident, 2usize) => {
        $crate::vtable!($class: $interface, Two)
    };
    ($class:ident: $interface:ident, 1usize) => {
        $crate::vtable!($class: $interface, One)
    };
    ($class:ident: $interface:ident, 0usize) => {
        $crate::vtable!($class: $interface, Zero)
    };
    ($class:ident: $interface:ident) => {
        $crate::vtable!($class: $interface, Zero)
    };
}
