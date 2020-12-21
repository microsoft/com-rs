com::interfaces! {
    #[uuid("12345678-1234-1234-0000-12345678ABCD")]
    pub unsafe interface IEmpty: com::interfaces::iunknown::IUnknown {}

    #[uuid("12345678-1234-1234-0001-12345678ABCD")]
    pub unsafe interface ISomething: com::interfaces::iunknown::IUnknown {
        fn DoSomething(&self, empty: &IEmpty);
    }
}

com::class! {
    #[no_class_factory]
    pub class EmptyClass: IEmpty {}
    impl IEmpty for _ {}
}

com::class! {
    #[no_class_factory]
    pub class SomethingClass: ISomething {}
    impl ISomething for _ {
        fn DoSomething(&self, _empty: &IEmpty) {}
    }
}

fn main() {
    let empty_instance = EmptyClass::allocate();
    let empty = empty_instance.query_interface::<IEmpty>().unwrap();

    let something_instance = SomethingClass::allocate();
    let something = something_instance.query_interface::<ISomething>().unwrap();

    unsafe {
        something.DoSomething(&empty);
    }
}
