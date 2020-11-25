com::interfaces! {
    #[uuid("12345678-1234-1234-1234-12345678ABCD")]
    pub unsafe interface ISomething: com::interfaces::iunknown::IUnknown {}
}

com::class! {
    pub class ClassOfZero: ISomething {
    }

    impl ISomething for SomeClass {}
}
com::class! {
    pub class ClassOfOne: ISomething {
        one: u32,
    }

    impl ISomething for SomeClass {}
}
com::class! {
    pub class ClassOfTwo: ISomething {
        one: u32,
        two: u32
    }

    impl ISomething for SomeClass {}
}

fn main() {}
