#![allow(clippy::needless_lifetimes)]

pub mod class;
pub mod interface;
mod utils;
#[cfg(test)]
mod tests;

pub use class::Class;
pub use interface::Interface;
pub use interface::Interfaces;
