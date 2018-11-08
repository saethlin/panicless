/// A continer backed by a Vec with a cursor that always points to a valid element,
/// and therefore it is always possible to get the current element.
/// The backing container must never be empty.
use vec::ChillVec as Vec;

pub struct CursorVec<T> {
    index: usize,
    vec: Vec<T>,
}

impl<T> CursorVec<T> {
    /// Construct a CursorVec from a single element
    #[no_panic]
    pub fn new(first: T) -> CursorVec<T> {
        let mut vec = Vec::new();
        vec.push(first);
        Self { index: 0, vec }
    }

    #[no_panic]
    pub fn get(&self) -> &T {
        unsafe { self.vec.get_unchecked(self.index) }
    }

    #[no_panic]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { self.vec.get_unchecked_mut(self.index) }
    }

    #[no_panic]
    pub fn next(&mut self) {
        self.index += 1;
        if self.index == self.vec.len() {
            self.index = 0;
        }
    }

    #[no_panic]
    pub fn prev(&mut self) {
        if self.index == 0 {
            self.index = self.vec.len() - 1;
        } else {
            // Doesn't matter what we use here, because it's always > 0
            self.index += 1;
        }
    }

    #[no_panic]
    pub fn get_first_mut(&mut self) -> &mut T {
        unsafe { self.vec.get_unchecked_mut(0) }
    }

    #[no_panic]
    pub fn push(&mut self, item: T) {
        self.vec.push(item)
    }

    #[no_panic]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }

    #[no_panic]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.vec.iter_mut()
    }

    #[no_panic]
    pub fn tell(&self) -> usize {
        self.index
    }

    #[no_panic]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[no_panic]
    pub fn sort_by_key<K, F>(&mut self, f: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.vec.as_mut_slice().sort_by_key(f);
    }
}
