#![allow(dead_code)]

use core::ptr::NonNull;

#[repr(C)]
pub struct Slab {
    memory: NonNull<u8>,
    object_size: usize,
    capacity: usize,
    free_count: usize,
    free_list_head: Option<NonNull<FreeObject>>,
}

#[repr(C)]
struct FreeObject {
    next: Option<NonNull<FreeObject>>,
}

impl Slab {
    pub const MIN_OBJECT_SIZE: usize = core::mem::size_of::<usize>();
    
    pub fn new(object_size: usize, slab_size: usize) -> Self {
        assert!(object_size >= Self::MIN_OBJECT_SIZE);
        assert!(slab_size >= object_size);
        
        let capacity = slab_size / object_size;
        
        Self {
            memory: NonNull::dangling(),
            object_size,
            capacity,
            free_count: 0,
            free_list_head: None,
        }
    }
    
    pub unsafe fn init(&mut self, memory: NonNull<u8>) {
        self.memory = memory;
        self.free_count = self.capacity;
        
        let mut current = memory.as_ptr();
        for i in 0..self.capacity {
            let obj = current as *mut FreeObject;
            (*obj).next = if i < self.capacity - 1 {
                Some(NonNull::new_unchecked(current.add(self.object_size) as *mut FreeObject))
            } else {
                None
            };
            current = current.add(self.object_size);
        }
        
        self.free_list_head = Some(NonNull::new_unchecked(memory.as_ptr() as *mut FreeObject));
    }
    
    pub unsafe fn alloc(&mut self) -> Option<NonNull<u8>> {
        if self.is_full() {
            return None;
        }
        
        let free_obj = self.free_list_head?;
        let next = (*free_obj.as_ptr()).next;
        
        self.free_list_head = next;
        self.free_count -= 1;
        
        Some(free_obj.cast())
    }
    
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>) {
        let obj = ptr.cast::<FreeObject>().as_ptr();
        (*obj).next = self.free_list_head;
        self.free_list_head = Some(NonNull::new_unchecked(obj));
        self.free_count += 1;
    }
    
    pub fn contains(&self, ptr: NonNull<u8>) -> bool {
        let start = self.memory.as_ptr() as usize;
        let end = start + (self.capacity * self.object_size);
        let addr = ptr.as_ptr() as usize;
        addr >= start && addr < end
    }
    
    pub fn free_count(&self) -> usize { self.free_count }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn object_size(&self) -> usize { self.object_size }
    pub fn is_full(&self) -> bool { self.free_count == 0 }
    pub fn is_empty(&self) -> bool { self.free_count == self.capacity }
}
