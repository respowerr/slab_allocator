use crate::slab::Slab;

pub struct SCache {
    partial: Option<&'static mut Slab>,
    full: Option<&'static mut Slab>,
    object_size: usize,
}
