mod base_absolute {
    com::interfaces! {
        #[uuid("12345678-1234-1234-1234-12345678ABCD")]
        pub unsafe interface IBaseAbsolute: com::interfaces::iunknown::IUnknown {}
    }
}

mod specific_absolute {
    com::interfaces! {
        #[uuid("12345678-1234-1234-1234-12345678ABCE")]
        unsafe interface ISpecificAbsolute: crate::base_absolute::IBaseAbsolute {}
    }
}

mod specific_relative {
    com::interfaces! {
        #[uuid("12345678-1234-1234-1234-12345678ABCE")]
        unsafe interface ISpecificRelative: super::base_absolute::IBaseAbsolute {}
    }
}

mod base_use {
    use com::interfaces::iunknown::IUnknown;

    com::interfaces! {
        #[uuid("12345678-1234-1234-1234-12345678ABCD")]
        pub unsafe interface IBaseUse: IUnknown {}
    }
}

mod specific_use {
    use crate::base_use::IBaseUse;

    com::interfaces! {
        #[uuid("12345678-1234-1234-1234-12345678ABCE")]
        unsafe interface ISpecificUse: IBaseUse {}
    }
}

fn main() {}
