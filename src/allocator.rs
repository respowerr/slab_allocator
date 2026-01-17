use crate::cache::SCache;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{NonNull, null_mut};
use spin::Mutex;

const CACHE_SIZES: [usize; 8] = [8, 16, 32, 64, 128, 256, 512, 1024];
const MAX_CACHE_SIZE: usize = 1024;
const HEAP_SIZE: usize = 1024 * 1024;

static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut HEAP_INDEX: usize = 0;

unsafe fn alloc_page() -> *mut u8 {
    let heap_ptr = core::ptr::addr_of_mut!(HEAP_MEMORY) as *mut u8;
    let current_ptr = heap_ptr.add(HEAP_INDEX);
    let align_offset = current_ptr.align_offset(4096);
    let start_index = HEAP_INDEX + align_offset;
    let end_index = start_index + 4096;
    if end_index > HEAP_SIZE {
        return null_mut();
    }
    HEAP_INDEX = end_index;
    heap_ptr.add(start_index)
}

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

    unsafe fn refill(&self, cache_idx: usize) -> bool {
        let page_ptr = alloc_page();
        if page_ptr.is_null() {
            return false;
        }

        let slab_ptr = page_ptr as *mut crate::slab::Slab;
        let slab_size = core::mem::size_of::<crate::slab::Slab>();
        let memory_start = page_ptr.add(slab_size);
        let available_memory = 4096 - slab_size;
        let object_size = CACHE_SIZES[cache_idx];

        core::ptr::write(slab_ptr, crate::slab::Slab::new(object_size, available_memory));
        (*slab_ptr).init(NonNull::new_unchecked(memory_start));

        let slab_ref = &mut *slab_ptr;
        self.caches[cache_idx].lock().insert(slab_ref);
        true
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

        drop(cache);

        if self.refill(cache_idx) {
            let mut cache = self.caches[cache_idx].lock();
            if let Some(ptr) = cache.alloc() {
                return ptr.as_ptr();
            }
        }

        null_mut()
    }

    /// # Safety
    /// `ptr` must have been allocated by this allocator
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let size = layout.size();
        let cache_idx = match self.get_cache_index(size) {
            Some(idx) => idx,
            None => return,
        };

        let mut cache = self.caches[cache_idx].lock();
        let _ = cache.dealloc(NonNull::new_unchecked(ptr));
    }
}
