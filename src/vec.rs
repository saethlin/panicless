use core::mem;
use core::mem::{align_of, size_of};
use core::slice::SliceIndex;
use core::{ptr, slice};
use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};

#[derive(Debug)]
pub struct ChillVec<T> {
    data: *mut T,
    length: usize,
    capacity: usize,
}

impl<T> ChillVec<T> {
    pub fn new() -> Self {
        Self {
            data: ptr::NonNull::dangling().as_ptr(),
            length: 0,
            capacity: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        // Attempting to allocate zero size is UB
        // Calling with_capacity(0) is probably a programming error but we're not about
        // failure here. Instead, we do the most unsurprising thing possible.
        if cap == 0 {
            return Self::new();
        }

        let data = unsafe {
            let layout = Layout::from_size_align_unchecked(cap * size_of::<T>(), align_of::<T>());
            let data = alloc(layout);
            if data.is_null() {
                handle_alloc_error(layout);
            }
            data as *mut T
        };
        Self {
            data,
            length: 0,
            capacity: cap,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }
        unsafe {
            // Special case for the first allocation
            if self.capacity == 0 {
                let layout = Layout::from_size_align_unchecked(
                    new_capacity * size_of::<T>(),
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
                    size_of::<T>() * self.capacity,
                    align_of::<T>(),
                );
                let new_alloc =
                    realloc(self.data as *mut u8, layout, new_capacity * size_of::<T>());
                if !new_alloc.is_null() {
                    self.data = new_alloc as *mut T;
                    self.capacity = new_capacity;
                } else {
                    handle_alloc_error(layout);
                }
            }
        }
    }

    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            self.reserve(4);
        } else if self.length == self.capacity {
            let new_capacity = self.capacity + self.capacity / 2;
            self.reserve(new_capacity)
        }
        unsafe {
            ptr::write(self.data.offset(self.length as isize), item);
        }
        self.length += 1;
    }

    pub unsafe fn get_unchecked<I>(&self, index: I) -> &<I as SliceIndex<[T]>>::Output
    where
        I: SliceIndex<[T]>,
    {
        self.as_slice().get_unchecked(index)
    }

    pub unsafe fn get_unchecked_mut(&self, index: usize) -> &mut T {
        mem::transmute(self.data.offset(index as isize))
    }

    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>,
    {
        self.as_slice().get(index)
    }

    pub fn get_mut(&self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            Some(unsafe { self.get_unchecked_mut(index) })
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.length) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.length) }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.as_mut_slice().iter_mut()
    }
}

impl<T: Copy> ChillVec<T> {
    pub fn extend_from_slice(&mut self, items: &[T]) {
        use core::cmp::max;
        let new_len = self.len() + items.len();
        if new_len > self.capacity() {
            let new_capacity = max(self.capacity() + self.capacity() / 2, new_len);
            self.reserve(new_capacity)
        }
        unsafe {
            ptr::copy_nonoverlapping(
                items.as_ptr(),
                self.data.offset(self.len() as isize),
                items.len(),
            );
        }
        self.length = new_len;
    }
}

impl<T> Drop for ChillVec<T> {
    fn drop(&mut self) {
        // If capacity is 0 no allocation was done and the pointer is dangling
        if self.capacity > 0 {
            unsafe {
                dealloc(
                    self.data as *mut u8,
                    Layout::from_size_align_unchecked(
                        size_of::<T>() * self.capacity,
                        align_of::<T>(),
                    ),
                );
            }
        }
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
        assert_eq!(v.capacity(), 0);

        v.reserve(2);
        assert!(v.capacity() >= 2);

        for i in 0..16 {
            v.push(i);
        }

        assert!(v.capacity() >= 16);

        v.push(16);

        assert!(v.capacity() >= 17)
    }
}
