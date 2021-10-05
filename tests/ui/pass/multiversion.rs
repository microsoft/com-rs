use com::interfaces::IUnknown;

com::interfaces! {
    #[uuid("00000000-0000-0000-0000-000000000001")]
    pub unsafe interface IFoo : IUnknown {
        fn do_the_foo(&self) -> i32;
    }

    #[uuid("00000000-0000-0000-0000-000000000002")]
    pub unsafe interface IFoo2 : IUnknown {
        fn do_the_foo(&self) -> i32;
    }
}

com::class! {
    pub class FooWithTwoVersions : IFoo2, IFoo {
    }

    impl IFoo for FooWithTwoVersions {
        fn do_the_foo(&self) -> i32 {
            // The V1 behavior is to return 100
            100
        }
    }

    impl IFoo2 for FooWithTwoVersions {
        fn do_the_foo(&self) -> i32 {
            // The V2 behavior is to return 200
            200
        }
    }
}

fn main() {
    unsafe {
        let foo_object = FooWithTwoVersions::allocate();

        let foo_v1 = foo_object.query_interface::<IFoo>().unwrap();
        assert_eq!(100, foo_v1.do_the_foo());

        let foo_v2 = foo_object.query_interface::<IFoo2>().unwrap();
        assert_eq!(200, foo_v2.do_the_foo());
    }
}
