#![cfg_attr(not(test), no_std)]

pub mod slab;
pub mod cache;
pub mod allocator;

#[cfg(test)]
mod tests;
