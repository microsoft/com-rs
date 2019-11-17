use com::interfaces::iunknown::IUnknown;
use com::*;
use std::fmt::Display;

#[com_interface("12345678-1234-1234-1234-12345678ABCD")]
trait IInterface: IUnknown {
    fn default_length(&self) -> usize;
    fn value_length(&self) -> usize;
}

#[co_class(implements(IInterface))]
pub struct CoClass<T: Default + Display> {
    value: T,
}

impl<T: Default + Display> IInterface for CoClass<T> {
    fn default_length(&self) -> usize {
        T::default().to_string().len()
    }
    fn value_length(&self) -> usize {
        self.value.to_string().len()
    }
}

fn main() {}
