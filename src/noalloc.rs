pub struct AtomAlloc<const SLOTS: usize, const FRAME_SIZE: usize> {
    pub atoms: [Option<Atom<FRAME_SIZE>>; SLOTS],
    pub head: Option<usize>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Atom<const FRAME_SIZE: usize> {
    pub start: usize,
    pub frame_count: usize,
}

impl<const SLOTS: usize, const FRAME_SIZE: usize> Default for AtomAlloc<SLOTS, FRAME_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SLOTS: usize, const FRAME_SIZE: usize> AtomAlloc<SLOTS, FRAME_SIZE> {
    pub const fn new() -> Self {
        Self {
            atoms: [const { None }; SLOTS],
            head: None,
        }
    }

    fn head(&self) -> &Option<Atom<FRAME_SIZE>> {
        // self.head should always point to either a valid atom or a none
        if let Some(index) = self.head {
            &self.atoms[index]
        } else {
            &None
        }
    }

    fn push(&mut self, atom: Atom<FRAME_SIZE>) {
        if atom.frame_count == 0 {
            // empty atom, do nothing
            return;
        }
        if let Some(index) = self.head {
            // if there's an atom at the head, add one and put an atom there
            self.atoms[index + 1] = Some(atom);
            self.head = Some(index + 1);
        } else {
            // no atom, add the first atom
            self.head = Some(0);
            self.atoms[0] = Some(atom);
        }
    }

    fn pop(&mut self) -> Option<Atom<FRAME_SIZE>> {
        if let Some(index) = self.head {
            // if there's an atom at the head...
            if index == 0 {
                // if the atom is the first one, unset head and return first atom
                self.head = None;
                self.atoms[0].take()
            } else {
                // take and subtract head
                self.head = Some(index - 1);
                self.atoms[index - 1].take()
            }
        } else {
            None
        }
    }

    /// # add_memory
    /// this function will add memory to the allocator, starting at `start`
    /// `frame_count` is the amount of frames that are available after `start`
    ///
    /// no alignment guarantees are made,
    /// so if you want alignment then do that before inserting the frames
    pub fn add_memory(&mut self, start: usize, frame_count: usize) {
        self.push(Atom { start, frame_count })
    }

    /// # allocate
    /// this function will find a free space of the given `frame_count`;
    /// the starting address will be returned, once again with no alignment guarantees
    /// will return `None` if there is not enough memory to allocate the given `frame_count`
    /// if there is enough time, it may be beneficial to call `defragment()` beforehand
    pub fn allocate(&mut self, frame_count: usize) -> Option<usize> {
        // is the head atom good enough?
        let top = self.pop()?;
        if top.frame_count >= frame_count {
            // allocate from the start
            self.push(Atom {
                start: top.start + (frame_count * FRAME_SIZE),
                frame_count: top.frame_count - frame_count,
            });
            Some(top.start)
        } else {
            // search all of the slots
            let mut search_stack = [const { None }; SLOTS];
            search_stack[0] = Some(top);
            let mut head = 0;

            let mut found = None;

            while found.is_none() {
                let top = self.pop()?;
                if top.frame_count >= frame_count {
                    found = Some(top);
                } else {
                    head += 1;
                    search_stack[head] = Some(top);
                }
            }

            // refill
            while head != 0 {
                let top = search_stack[head].take()?;
                head -= 1;
                self.push(top);
            }

            if let Some(found) = found {
                // this is the atom we can allocate from
                self.push(Atom {
                    start: found.start + (frame_count * FRAME_SIZE),
                    frame_count: found.frame_count - frame_count,
                });
                Some(found.start)
            } else {
                None
            }
        }
    }

    /// # deallocate
    /// this function is a mirror of `add_memory()`, you must provide the amount of frames
    /// allocated, it won't work with a raw memory length
    pub fn deallocate(&mut self, start: usize, frame_count: usize) {
        self.add_memory(start, frame_count);
    }

    /// # defragment
    /// this function will rearrange the atoms available in the allocator so that they
    /// follow each other, and merge touching atoms
    ///
    /// this function does not check for overlapping atoms
    pub fn defragment(&mut self) {
        if let Some(head) = self.head {
            if head == 0 {
                return;
            }
            // first we will sort the atoms, i'm just gonna use insertion sort cause its simple
            let mut idx = 1;
            while idx <= head {
                let atom = self.atoms[idx].take().expect("None before head!");
                // go back through and insert the atom there
                let mut idx2 = idx;
                while idx2 > 0 {
                    idx2 -= 1;
                    // fixme: this can be optimized using std::mem::swap
                    // take the previous atom
                    let prev_atom = self.atoms[idx2].take().expect("None before head!");
                    // if the previous atom is greater, place current before it
                    if prev_atom.start > atom.start {
                        self.atoms[idx2 + 1] = Some(prev_atom);
                    } else {
                        // place atom here
                        self.atoms[idx2 + 1] = Some(prev_atom);
                        self.atoms[idx2] = Some(atom);
                        break;
                    }
                    if idx2 == 0 {
                        self.atoms[idx2] = Some(atom);
                        break;
                    }
                }
                idx += 1;
            }

            // should be sorted now, merge touching atoms
            let mut stack = [const { None }; SLOTS];
            
            // clippy, shut up
            #[allow(clippy::needless_range_loop)]
            for i in 0..=head {
                stack[head - i] = self.atoms[i].take()
            }
            
            let mut idx = head;
            let mut idx2 = 0;
            let mut atom = stack[head].take().expect("None before head!");
            loop {
                // take next atom
                if idx == 0 {
                    self.atoms[idx2] = Some(atom);
                    break;
                }
                let next = stack[idx - 1].take().expect("None before head!");
                if atom.start + (atom.frame_count * FRAME_SIZE) >= next.start {
                    // merge
                    atom.frame_count += next.frame_count;
                } else {
                    // cannot merge anymore, push and take
                    self.atoms[idx2] = Some(atom);
                    idx2 += 1;
                    atom = next;
                }
                idx -= 1;
            }
            self.head = Some(idx2);
        }
    }
}
