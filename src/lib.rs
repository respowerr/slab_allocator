#![no_std]
#![feature(allocator_api)]
#![feature(const_mut_refs)]

pub mod slab;
pub mod cache;
pub mod allocator;

#[cfg(test)]
mod tests;

pub use allocator::SlabAllocator;
