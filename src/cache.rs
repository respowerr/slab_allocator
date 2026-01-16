use crate::slab::Slab;
use core::ptr::NonNull;

pub struct SCache {
    partial: Option<&'static mut Slab>,
    full: Option<&'static mut Slab>,
    object_size: usize,
}

impl SCache {
    pub const fn new(object_size: usize) -> Self {
        Self {
            partial: None,
            full: None,
            object_size,
        }
    }

    pub fn insert(&mut self, slab: &'static mut Slab) {
        assert_eq!(slab.object_size(), self.object_size);
        assert!(!slab.is_full());
        slab.next = self.partial.take();
        self.partial = Some(slab);
    }

    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        let slab = self.partial.as_mut()?;
        let ptr: Option<NonNull<u8>> = unsafe { slab.alloc() };

        if ptr.is_some() && slab.is_full() {
            let full_slab = self.partial.take().unwrap();
            self.partial = full_slab.next.take();
            full_slab.next = self.full.take();
            self.full = Some(full_slab);
        }

        ptr
    }
}
