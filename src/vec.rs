use std::slice::SliceIndex;

#[derive(Debug)]
pub struct ChillVec<T> {
    data: *mut T,
    length: usize,
    capacity: usize,
}

impl<T> ChillVec<T> {
    #[no_panic]
    pub fn new() -> Self {
        Self {
            data: std::ptr::NonNull::dangling().as_ptr(),
            length: 0,
            capacity: 0,
        }
    }

    #[no_panic]
    pub fn len(&self) -> usize {
        self.length
    }

    #[no_panic]
    pub fn reserve(&mut self, new_capacity: usize) {
        use std::alloc::{alloc, handle_alloc_error, realloc, Layout};
        use std::mem::{align_of, size_of};
        if new_capacity <= self.capacity {
            return;
        }
        unsafe {
            // Special case for the first allocation
            if self.capacity == 0 {
                let layout = Layout::from_size_align_unchecked(
                    size_of::<T>().saturating_mul(new_capacity),
                    align_of::<T>(),
                );
                let new_alloc = alloc(layout);
                if !new_alloc.is_null() {
                    self.data = new_alloc as *mut T;
                    self.capacity = new_capacity;
                } else {
                    handle_alloc_error(layout);
                }
            } else {
                // Grow by reallocating
                let layout = Layout::from_size_align_unchecked(
                    size_of::<T>().saturating_mul(self.capacity),
                    align_of::<T>(),
                );
                let new_alloc = realloc(self.data as *mut u8, layout, new_capacity);
                if !new_alloc.is_null() {
                    self.data = new_alloc as *mut T;
                    self.capacity = new_capacity;
                } else {
                    handle_alloc_error(layout);
                }
            }
        }
    }

    #[no_panic]
    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            self.reserve(2);
        } else if self.length == self.capacity {
            let new_capacity = self.capacity.saturating_mul(2);
            self.reserve(new_capacity)
        }
        unsafe {
            std::ptr::write(self.data.offset(self.length as isize), item);
        }
        self.length = self.length.saturating_add(1);
    }

    #[no_panic]
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        std::mem::transmute(self.data.offset(index as isize))
    }

    #[no_panic]
    pub unsafe fn get_unchecked_mut(&self, index: usize) -> &mut T {
        std::mem::transmute(self.data.offset(index as isize))
    }

    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>,
    {
        self.as_slice().get(index)
    }

    #[no_panic]
    pub fn get_mut(&self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            Some(unsafe { self.get_unchecked_mut(index) })
        }
    }

    #[no_panic]
    pub fn as_slice(&self) -> &[T] {
        use std::slice::from_raw_parts;
        unsafe {
            if self.data.is_null() {
                from_raw_parts(std::ptr::NonNull::dangling().as_ptr(), self.length)
            } else {
                from_raw_parts(self.data, self.length)
            }
        }
    }

    #[no_panic]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        use std::slice::from_raw_parts_mut;
        unsafe {
            if self.data.is_null() {
                from_raw_parts_mut(std::ptr::NonNull::dangling().as_ptr(), self.length)
            } else {
                from_raw_parts_mut(self.data, self.length)
            }
        }
    }

    #[no_panic]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.as_slice().iter()
    }

    #[no_panic]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.as_slice_mut().iter_mut()
    }

    #[no_panic]
    pub fn sort_by_key<K, F>(&mut self, key: F)
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.as_slice_mut().sort_by_key(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut vec = ChillVec::new();
        vec.push(1337);
        assert_eq!(vec.get(0), Some(&1337));
    }

    #[test]
    fn test_reserve() {
        let mut v = ChillVec::new();
        assert_eq!(v.capacity, 0);

        v.reserve(2);
        assert!(v.capacity >= 2);

        for i in 0..16 {
            v.push(i);
        }

        assert!(v.capacity >= 16);

        v.push(16);

        assert!(v.capacity >= 17)
    }
}
