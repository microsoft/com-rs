pub trait Offset {
    const VALUE: usize;
}

macro_rules! declare_offset {
    ($($name:ident => $value:literal),*) => {
        $(
            pub struct $name;

            impl Offset for $name {
                const VALUE: usize = $value;
            }
        )*
    };
}

declare_offset!(Zero => 0, One => 1, Two => 2, Three => 3, Four => 4);
