//! This crate demonstrates that COM can be used in `no_std` contexts.
#![no_std]

use com::interfaces::IUnknown;

com::interfaces! {
    #[uuid("b05271fc-4a1d-45ae-8f7a-c5f2dc0cde88")]
    pub unsafe interface IBike : IUnknown {
        pub fn ride(&self);
    }

    #[uuid("2015c764-184c-4f50-86cf-a00e831658e1")]
    pub unsafe interface ICar : IUnknown {
        pub fn drive(&self);
    }
}

com::class! {
    pub class HybridVehicle : IBike, ICar {}

    impl IBike for HybridVehicle {
        fn ride(&self) {}
    }

    impl ICar for HybridVehicle {
        fn drive(&self) {}
    }
}
