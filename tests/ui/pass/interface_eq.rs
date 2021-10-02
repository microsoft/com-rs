use com::interfaces::IUnknown;

com::interfaces! {
    #[uuid("00000000-0000-0000-0000-000000000001")]
    unsafe interface IFoo: IUnknown {}
}

com::class! {
    class SimpleClass: IFoo {
    }

    impl IFoo for SimpleClass {}
}

fn main() {
    let instance1 = SimpleClass::allocate();
    let instance1_as_foo = instance1.query_interface::<IFoo>().unwrap();
    let instance1_as_foo_again = instance1.query_interface::<IFoo>().unwrap();
    assert_eq!(instance1_as_foo, instance1_as_foo_again);

    let instance2 = SimpleClass::allocate();
    let instance2_as_foo = instance2.query_interface::<IFoo>().unwrap();
    assert_ne!(instance1_as_foo, instance2_as_foo);
}
