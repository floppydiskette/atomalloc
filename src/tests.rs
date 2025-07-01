use crate::noalloc::{Atom, AtomAlloc};

#[test]
fn noalloc() {
    let mut alloc: AtomAlloc<16, 1> = AtomAlloc::default();
    
    assert_eq!(alloc.head, None);
    
    alloc.add_memory(1024, 64); // [(1024, 64)...]
    assert_eq!(alloc.atoms[0], Some(Atom { start: 1024, frame_count: 64 }));
    assert_eq!(alloc.head, Some(0));
    
    // allocation
    
    let start = alloc.allocate(10); // [(1034, 54)...]
    assert_eq!(alloc.atoms[0], Some(Atom { start: 1034, frame_count: 54 }));
    assert_eq!(start, Some(1024));
    assert_eq!(alloc.head, Some(0));
    
    // deallocation
    
    alloc.deallocate(1024, 10); // [(1034, 54), (1024, 10)...]
    assert_eq!(alloc.atoms[0], Some(Atom { start: 1034, frame_count: 54 }));
    assert_eq!(alloc.atoms[1], Some(Atom { start: 1024, frame_count: 10 }));
    assert_eq!(alloc.head, Some(1));
    
    // defragmentation
    
    alloc.defragment(); // [(1024, 64)...]
    assert_eq!(alloc.atoms[0], Some(Atom { start: 1024, frame_count: 64 }));
    assert_eq!(alloc.head, Some(0));
}