#[allow(clippy::module_inception)]
mod class;
mod class_constructor;
mod class_factory;
mod iunknown_impl;
#[cfg(test)]
mod tests;

pub use class::Class;
