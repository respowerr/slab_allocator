# Linux Kernel SLUB Allocator in no_std Rust

ESGI - 4SI

## 1. Introduction
The SLUB allocator is the default memory allocator in the Linux kernel since version 2.6.23. It replaced the original SLAB allocator to reduce metadata overhead and improve performance on large systems. Understanding its layout is crucial for kernel exploitation techniques like Heap Overflows and Use-After-Free.

## 2. High-Level Architecture
Unlike the Buddy Allocator (which manages physical pages of 4096 bytes), SLUB manages objects of specific sizes (caches).

### Key Components:
- **kmem_cache**: Represents a "bucket" for objects of a specific size (e.g., `kmalloc-32`, `kmalloc-64`).
- **slab**: A contiguous chunk of memory (usually one page, 4KB) containing multiple objects.
- **freelist**: A linked list pointing to the next free object within a slab.

## 3. Physical Layout & Metadata
SLUB is designed to eliminate external metadata headers (unlike the old SLAB).
- The `struct page` (in `vmemmap`) holds all the metadata for a slab.
- The actual slab memory contains **only** the objects, packed tightly.

### The Freelist Obfuscation (Security Mitigation)
To prevent easy exploitation (like overwriting the `next` pointer in a UAF scenario), the kernel obfuscates the freelist pointer since kernel 4.14 (approx).
**Formula:**
`stored_ptr = current_ptr ^ random_secret ^ next_ptr`

An attacker must leak the `random_secret` (cookie) or the kernel base address to successfully hijack the freelist control flow.

## 4. Allocation Logic (The "Fast Path")
SLUB attempts to be lockless for performance.
1. **Per-CPU Cache**: Each CPU has a dedicated active slab (`c->page`).
2. If `c->page` has free objects (`c->freelist` is not NULL), it returns the object immediately.
3. This is the most dangerous path for exploitation because it lacks heavy checks.

## 5. Exploitation Primitives
The design of SLUB enables several attack vectors:

### A. Heap Overflow / Slub Overflow
If an attacker can overflow an object in `kmalloc-64`, they write into the *next* adjacent object in memory.
- **Target**: Overwriting function pointers or objects with `ops` structures (e.g., `seq_operations`, `tty_struct`).
- **Goal**: Redirect control flow to ROP chains.

### B. Use-After-Free (UAF)
1. Allocate an object A.
2. Free object A (it returns to the freelist).
3. Allocate object B (victim object).
4. If the attacker still holds a reference to A, they can read/write B.

### C. Freelist Corruption
By overflowing into a free object, an attacker can modify the `next` pointer (the "freelist" pointer).
- **Goal**: Force the allocator to return a pointer to an arbitrary memory address (e.g., return a pointer to the kernel stack or creds structure).
- **Hurdle**: Requires bypassing the `CONFIG_SLAB_FREELIST_HARDENED` (pointer obfuscation).

## 6. Cross-Cache Attacks
SLUB merges caches with similar properties (aliasing).
- `kmalloc-128` might share the same underlying slab as a dedicated structure cache of size 120.
- An attacker can free an object in one cache and reclaim it via a different cache type to cause type confusion.
