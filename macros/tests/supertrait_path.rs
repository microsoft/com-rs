mod base_absolute {
    #[com::com_interface("12345678-1234-1234-1234-12345678ABCD")]
    pub trait IBaseAbsolute: com::interfaces::iunknown::IUnknown {}
}

mod specific_absolute {
    #[com::com_interface("12345678-1234-1234-1234-12345678ABCE")]
    trait ISpecificAbsolute: crate::base_absolute::IBaseAbsolute {}
}

mod specific_relative {
    #[com::com_interface("12345678-1234-1234-1234-12345678ABCE")]
    trait ISpecificRelative: super::base_absolute::IBaseAbsolute {}
}

mod base_use {
    use com::interfaces::iunknown::IUnknown;

    #[com::com_interface("12345678-1234-1234-1234-12345678ABCD")]
    pub trait IBaseUse: IUnknown {}
}

mod specific_use {
    use crate::base_use::IBaseUse;

    #[com::com_interface("12345678-1234-1234-1234-12345678ABCE")]
    trait ISpecificUse: IBaseUse {}
}

fn main() {}
