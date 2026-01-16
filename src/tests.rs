#![cfg(test)]

use crate::slab::Slab;
use core::ptr::NonNull;
use std::alloc::{GlobalAlloc, Layout, System};

#[test]
fn test_slab_creation() {
    let slab = Slab::new(64, 4096);
    assert_eq!(slab.object_size(), 64);
    assert_eq!(slab.capacity(), 64);
}

#[test]
fn test_slab_init() {
    let mut slab = Slab::new(64, 4096);
    
    unsafe {
        let layout = Layout::from_size_align_unchecked(4096, 8);
        let ptr = System.alloc(layout);
        assert!(!ptr.is_null());
        
        slab.init(NonNull::new_unchecked(ptr));
        
        assert_eq!(slab.free_count(), 64);
        assert!(slab.is_empty());
        assert!(!slab.is_full());
        
        System.dealloc(ptr, layout);
    }
}

#[test]
#[should_panic]
fn test_invalid_object_size() {
    let _ = Slab::new(4, 4096);
}
