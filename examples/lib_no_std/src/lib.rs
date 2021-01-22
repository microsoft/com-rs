//! This demonstrates that COM can be used in a `#![no_std]` environment.

#![no_std]

use com::interfaces::IUnknown;

com::interfaces! {
    #[uuid("f6681e01-0a99-47ce-8c04-71e82742c103")]
    pub unsafe interface IFoo : IUnknown {
        pub fn do_the_foo(&self, x: u32) -> u32;
    }
}

pub fn hello_world(foo: &IFoo) -> u32 {
    unsafe { foo.do_the_foo(42) }
}

pub fn maybe_hello_world(unknown: &IUnknown) -> Option<IFoo> {
    if let Some(foo) = unknown.query_interface::<IFoo>() {
        unsafe {
            foo.do_the_foo(100);
        }
        Some(foo)
    } else {
        None
    }
}
