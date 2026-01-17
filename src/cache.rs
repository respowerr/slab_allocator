use crate::slab::Slab;
use core::ptr::NonNull;

pub struct CacheStats {
    pub partial_slabs: usize,
    pub full_slabs: usize,
    pub total_objects: usize,
    pub used_objects: usize,
}

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

    /// # Safety
    /// `ptr` must belong to a slab in this cache
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>) -> bool {
        let mut current = self.partial.as_deref_mut();
        while let Some(slab) = current {
            if slab.contains(ptr) {
                slab.dealloc(ptr);
                return true;
            }
            current = slab.next.as_deref_mut();
        }

        let mut current = self.full.as_deref_mut();
        while let Some(slab) = current {
            if slab.contains(ptr) {
                slab.dealloc(ptr);
                return true;
            }
            current = slab.next.as_deref_mut();
        }
        false
    }

    pub fn stats(&self) -> CacheStats {
        let mut partial_count = 0;
        let mut full_count = 0;
        let mut total_objects = 0;
        let mut used_objects = 0;

        let mut current = self.partial.as_ref();
        while let Some(slab) = current {
            partial_count += 1;
            total_objects += slab.capacity();
            used_objects += slab.used_count();
            current = slab.next.as_ref();
        }

        let mut current = self.full.as_ref();
        while let Some(slab) = current {
            full_count += 1;
            total_objects += slab.capacity();
            used_objects += slab.used_count();
            current = slab.next.as_ref();
        }

        CacheStats {
            partial_slabs: partial_count,
            full_slabs: full_count,
            total_objects,
            used_objects,
        }
    }
}
