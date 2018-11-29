use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem::{align_of, size_of};
use std::{ptr, slice};

#[derive(Debug)]
pub struct ChillVec<T> {
    data: *mut T,
    length: usize,
    capacity: usize,
}

// Rust specifies that pointers cannot be offset by more than isize::MAX in bytes
// so we abort on attempts to allocate more than that amount of memory, because
// to do otherwise requires a bounds-check on every element access and produces
// unreachable data.
// If we omit this check there would be possible UB when accessing an
// element, and could actually happen today on a 32-bit platform
#[inline]
fn abort_if_alloc_too_large<T>(capacity: usize) {
    if capacity
        .saturating_mul(size_of::<T>())
        .saturating_add(align_of::<T>())
        > isize::max_value() as usize
    {
        unsafe {
            handle_alloc_error(Layout::from_size_align_unchecked(
                capacity * size_of::<T>(),
                align_of::<T>(),
            ));
        }
    }
}

impl<T> Default for ChillVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ChillVec<T> {
    /// ```
    /// let vec = panicless::ChillVec::<usize>::new();
    /// assert!(vec.len() == 0);
    /// assert!(vec.capacity() == 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        assert!(size_of::<T>() > 0);
        Self {
            data: ptr::NonNull::dangling().as_ptr(),
            length: 0,
            capacity: 0,
        }
    }

    /// ```
    /// let vec = panicless::ChillVec::<usize>::with_capacity(20);
    /// assert!(vec.len() == 0);
    /// // Space for more than 20 elements may be allocated if there is no additional cost
    /// assert!(vec.capacity() >= 20);
    /// ```
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        // Calling with_capacity(0) is probably a programming error but we're not about
        // failure here. Instead, we do the most unsurprising thing possible.

        assert!(size_of::<T>() > 0);

        if cap == 0 {
            return Self::new();
        }

        abort_if_alloc_too_large::<T>(cap);

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

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// ```
    /// let mut vec = panicless::ChillVec::<usize>::new();
    /// assert!(vec.capacity() == 0);
    /// vec.reserve(20);
    /// // Space for more than 20 elements may be allocated if there is no additional cost
    /// assert!(vec.capacity() >= 20);
    /// ```
    #[inline]
    pub fn reserve(&mut self, new_capacity: usize) {
        // Save silly users from themselves
        // This has a small additional cost on every call, but may save an entire allocation
        // If this function is made private we could remove the check
        if new_capacity <= self.capacity {
            return;
        }

        abort_if_alloc_too_large::<T>(new_capacity);

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

    #[inline]
    pub fn push(&mut self, item: T) {
        if self.length == self.capacity {
            let new_capacity = self.capacity + self.capacity / 2 + 1;
            self.reserve(new_capacity)
        }

        unsafe {
            ptr::write(self.data.add(self.length), item);
        }
        self.length += 1;
    }
}

impl<T: Copy> ChillVec<T> {
    #[inline]
    pub fn extend_from_slice(&mut self, items: &[T]) {
        use std::cmp::max;
        let new_len = self.len() + items.len();
        if new_len > self.capacity() {
            let new_capacity = max(self.capacity() + self.capacity() / 2 + 1, new_len);
            self.reserve(new_capacity)
        }
        unsafe {
            ptr::copy_nonoverlapping(items.as_ptr(), self.data.add(self.len()), items.len());
        }
        self.length = new_len;
    }
}

impl<T> Drop for ChillVec<T> {
    #[inline]
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

impl<T> std::ops::Deref for ChillVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data as *const T, self.length) }
    }
}

impl<T> std::ops::DerefMut for ChillVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.length) }
    }
}

impl<'a, T> IntoIterator for &'a ChillVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> std::slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ChillVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> std::slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push() {
        let mut vec = ChillVec::new();
        assert_eq!(vec.len(), 0);

        vec.push(1337);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(&1337));

        vec.push(1);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(1), Some(&1));
    }

    #[test]
    fn index_range() {
        let mut vec = ChillVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);

        assert_eq!(vec[..1], [1]);
        assert_eq!(vec[..2], [1, 2]);
        assert_eq!(vec[2..4], [3, 4]);
    }

    #[test]
    fn iterate() {
        let mut vec = ChillVec::new();
        vec.extend_from_slice(&[1, 2, 3, 4]);
        let mut it = vec.iter();
        assert_eq!(it.next(), Some(&1));
        assert_eq!(it.next(), Some(&2));
        assert_eq!(it.next(), Some(&3));
        assert_eq!(it.next(), Some(&4));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);

        for it in &vec {
            assert!(*it > 0 && *it < 5);
        }

        assert_eq!(vec.iter().count(), 4);
    }

    #[test]
    fn reserve() {
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
