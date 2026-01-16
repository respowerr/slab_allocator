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

#[test]
fn test_alloc_dealloc() {
    let mut slab = Slab::new(64, 4096);
    
    unsafe {
        let layout = Layout::from_size_align_unchecked(4096, 8);
        let ptr = System.alloc(layout);
        slab.init(NonNull::new_unchecked(ptr));
        
        let obj1 = slab.alloc().unwrap();
        assert_eq!(slab.free_count(), 63);
        
        let obj2 = slab.alloc().unwrap();
        assert_eq!(slab.free_count(), 62);
        
        slab.dealloc(obj1);
        assert_eq!(slab.free_count(), 63);
        
        slab.dealloc(obj2);
        assert_eq!(slab.free_count(), 64);
        assert!(slab.is_empty());
        
        System.dealloc(ptr, layout);
    }
}

#[test]
fn test_alloc_full_slab() {
    let mut slab = Slab::new(64, 256);
    
    unsafe {
        let layout = Layout::from_size_align_unchecked(256, 8);
        let ptr = System.alloc(layout);
        slab.init(NonNull::new_unchecked(ptr));
        
        for _ in 0..4 {
            assert!(slab.alloc().is_some());
        }
        
        assert!(slab.is_full());
        assert!(slab.alloc().is_none());
        
        System.dealloc(ptr, layout);
    }
}

#[test]
fn test_contains() {
    let mut slab = Slab::new(64, 4096);
    
    unsafe {
        let layout = Layout::from_size_align_unchecked(4096, 8);
        let ptr = System.alloc(layout);
        slab.init(NonNull::new_unchecked(ptr));
        
        let obj = slab.alloc().unwrap();
        assert!(slab.contains(obj));
        
        let external = NonNull::new_unchecked(1 as *mut u8);
        assert!(!slab.contains(external));
        
        System.dealloc(ptr, layout);
    }
}
