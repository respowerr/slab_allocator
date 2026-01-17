use crate::cache::SCache;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

const CACHE_SIZES: [usize; 8] = [8, 16, 32, 64, 128, 256, 512, 1024];
const MAX_CACHE_SIZE: usize = 1024;

pub struct SlabAllocator {
    caches: [Mutex<SCache>; 8],
}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            caches: [
                Mutex::new(SCache::new(8)),
                Mutex::new(SCache::new(16)),
                Mutex::new(SCache::new(32)),
                Mutex::new(SCache::new(64)),
                Mutex::new(SCache::new(128)),
                Mutex::new(SCache::new(256)),
                Mutex::new(SCache::new(512)),
                Mutex::new(SCache::new(1024)),
            ],
        }
    }

    fn get_cache_index(&self, size: usize) -> Option<usize> {
        for (i, &cache_size) in CACHE_SIZES.iter().enumerate() {
            if size <= cache_size {
                return Some(i);
            }
        }
        None
    }

    pub fn object_size_for(&self, layout: Layout) -> Option<usize> {
        let size = layout.size();
        if size > MAX_CACHE_SIZE {
            return None;
        }
        for &cache_size in CACHE_SIZES.iter() {
            if size <= cache_size {
                return Some(cache_size);
            }
        }
        None
    }
}

unsafe impl GlobalAlloc for SlabAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        
        if size > MAX_CACHE_SIZE {
            return null_mut();
        }

        let cache_idx = match self.get_cache_index(size) {
            Some(idx) => idx,
            None => return null_mut(),
        };

        let mut cache = self.caches[cache_idx].lock();

        if let Some(ptr) = cache.alloc() {
            return ptr.as_ptr();
        }

        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }
}
