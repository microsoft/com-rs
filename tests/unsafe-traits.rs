// This is a compile-test to ensure the proc macro can handle unsafe traits.

use com::interfaces::iunknown::IUnknown;
use winapi::shared::winerror::NOERROR;

#[com::com_interface(12345678-1234-1234-1234-12345678ABCD)]
unsafe trait IUnsafe: IUnknown {
    fn method(&self);
}

#[com::co_class(implements(IUnsafe))]
struct CoClass {
    data: usize,
}

impl CoClass {
    fn new() -> Box<CoClass> {
        CoClass::allocate(0)
    }
}

unsafe impl IUnsafe for CoClass {
    fn method(&self) {}
}
