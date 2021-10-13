#![allow(clippy::needless_lifetimes)]
#![allow(clippy::upper_case_acronyms)]

pub mod class;
pub mod interface;
#[cfg(test)]
mod test_utils;
mod utils;

pub use class::Class;
pub use interface::Interface;
pub use interface::Interfaces;
