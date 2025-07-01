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

#[test]
fn noalloc_irl() {
    let mut alloc: AtomAlloc<1024, 4096> = AtomAlloc::default();

    alloc.add_memory(0x51000, 2);
    alloc.add_memory(0x54000, 72);
    alloc.add_memory(0x100000, 259887);
    alloc.add_memory(0x3f925000, 1);
    alloc.add_memory(0x3fed3000, 15);
    
    let mut first = alloc.allocate(1);
    if first.is_none() {
        alloc.defragment();
        first = alloc.allocate(1);
    }
    assert!(first.is_some());
    
    let mut second = alloc.allocate(1);
    if second.is_none() {
        alloc.defragment();
        second = alloc.allocate(1);
    }
    assert!(second.is_some());
    
    let mut third = alloc.allocate(1);
    if third.is_none() {
        alloc.defragment();
        third = alloc.allocate(1);
    }
    assert!(third.is_some());
}