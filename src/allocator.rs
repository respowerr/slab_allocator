use crate::cache::SCache;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

const CACHE_SIZES: [usize; 8] = [8, 16, 32, 64, 128, 256, 512, 1024];
const MAX_CACHE_SIZE: usize = 1024;

pub struct SlabAllocator {
    caches: [SCache; 8],
}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            caches: [
                SCache::new(8),
                SCache::new(16),
                SCache::new(32),
                SCache::new(64),
                SCache::new(128),
                SCache::new(256),
                SCache::new(512),
                SCache::new(1024),
            ],
        }
    }

    fn select_cache(&mut self, size: usize) -> Option<&mut SCache> {
        for (i, &cache_size) in CACHE_SIZES.iter().enumerate() {
            if size <= cache_size {
                return Some(&mut self.caches[i]);
            }
        }
        None
    }

    pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let size = layout.size();
        if size > MAX_CACHE_SIZE {
            return None;
        }
        
        let cache = self.select_cache(size)?;
        cache.alloc()
    }

    pub fn object_size_for(&self, layout: Layout) -> Option<usize> {
        let size = layout.size();
        CACHE_SIZES.iter().find(|&&cs| size <= cs).copied()
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
