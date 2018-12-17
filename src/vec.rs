use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem::{align_of, size_of};
use std::num::NonZeroUsize;
use std::ptr::NonNull;
use std::{ptr, slice};

#[inline]
fn alloc_or_abort<T>(n_elements: NonZeroUsize) -> NonNull<T> {
    unsafe {
        let layout =
            Layout::from_size_align_unchecked(n_elements.get() * size_of::<T>(), align_of::<T>());

        // Rust specifies that pointers cannot be offset by more than isize::MAX in bytes
        // so we abort on attempts to allocate more than that amount of memory, because
        // to do otherwise requires a bounds-check on every element access and produces
        // unreachable data.
        // If we omit this check there would be possible UB when accessing an
        // element, and could actually happen today on a 32-bit platform
        if layout.size() > isize::max_value() as usize {
            handle_alloc_error(layout);
        }

        NonNull::new(alloc(layout) as *mut T).unwrap_or_else(|| handle_alloc_error(layout))
    }
}

#[inline]
fn realloc_or_abort<T>(
    ptr: NonNull<T>,
    previous_size: NonZeroUsize,
    new_size: NonZeroUsize,
) -> NonNull<T> {
    unsafe {
        let old_layout = Layout::from_size_align_unchecked(
            previous_size.get() * size_of::<T>(),
            align_of::<T>(),
        );

        if old_layout.size() > isize::max_value() as usize {
            handle_alloc_error(old_layout);
        }

        NonNull::new(realloc(
            ptr.cast().as_ptr(),
            old_layout,
            new_size.get() * size_of::<T>(),
        ) as *mut T)
        .unwrap_or_else(|| {
            handle_alloc_error(Layout::from_size_align_unchecked(
                new_size.get(),
                align_of::<T>(),
            ))
        })
    }
}

#[derive(Debug)]
pub struct ChillVec<T> {
    data: NonNull<T>,
    length: usize,
    capacity: usize,
}

impl<T> Default for ChillVec<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for ChillVec<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        // This is not an optimization, it's required
        // The layout provided to alloc must have non-zero size
        let data = match NonZeroUsize::new(self.length) {
            Some(n) => alloc_or_abort(n),
            None => return Self::new(),
        };

        unsafe {
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), data.as_ptr(), self.length);
        };

        Self {
            length: self.length,
            capacity: self.length,
            data,
        }
    }
}

impl<T> ChillVec<T> {
    /// ```
    /// # use panicless::ChillVec;
    /// let vec = ChillVec::<usize>::new();
    /// assert!(vec.len() == 0);
    /// assert!(vec.capacity() == 0);
    /// ```
    #[inline]
    pub fn new() -> Self {
        assert!(size_of::<T>() > 0);
        Self {
            data: NonNull::dangling(),
            length: 0,
            capacity: 0,
        }
    }

    /// ```
    /// # use panicless::ChillVec;
    /// let vec = ChillVec::<usize>::with_capacity(20);
    /// assert!(vec.len() == 0);
    /// // Space for more than 20 elements may be allocated if there is no additional cost
    /// assert!(vec.capacity() >= 20);
    /// ```
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        assert!(size_of::<T>() > 0);

        let data = match NonZeroUsize::new(cap) {
            Some(n) => alloc_or_abort(n),
            None => return Self::new(),
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
    /// # use panicless::ChillVec;
    /// let mut vec = ChillVec::<usize>::new();
    /// assert!(vec.capacity() == 0);
    /// vec.reserve(20);
    /// // Space for more than 20 elements may be allocated
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

        let new_capacity = match NonZeroUsize::new(new_capacity) {
            Some(n) => n,
            None => return,
        };

        self.data = match NonZeroUsize::new(self.capacity) {
            None => alloc_or_abort(new_capacity),
            Some(old_capacity) => realloc_or_abort(self.data, old_capacity, new_capacity),
        };

        // In either case, we have succeeded
        self.capacity = new_capacity.get();
    }

    /// ```
    /// # use panicless::ChillVec;
    /// let mut vec = ChillVec::new();
    /// vec.push(0);
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec[0], 0);
    /// assert_eq!(vec[1], 1);
    /// assert_eq!(vec[2], 2);
    /// ```
    #[inline]
    pub fn push(&mut self, item: T) {
        if self.length == self.capacity {
            let new_capacity = self.capacity + self.capacity / 2 + 1;
            self.reserve(new_capacity)
        }

        unsafe {
            ptr::write(self.data.as_ptr().add(self.length), item);
        }
        self.length += 1;
    }

    // TODO This is possibly wrong, RawVec has a bajillion checks
    pub fn shrink_to_fit(&mut self) {
        if self.length > 0 && self.capacity > self.length {
            unsafe {
                let old_size = size_of::<T>() * self.capacity;
                let new_size = size_of::<T>() * self.length;
                let align = align_of::<T>();
                let old_layout = Layout::from_size_align_unchecked(old_size, align);

                self.data = NonNull::new(
                    realloc(self.data.cast().as_ptr(), old_layout, new_size) as *mut T
                )
                .unwrap_or_else(|| {
                    handle_alloc_error(Layout::from_size_align_unchecked(new_size, align))
                });
                self.capacity = self.length;
            }
        }
    }
}

impl<T: Copy> ChillVec<T> {
    #[inline]
    pub fn extend_from_slice(&mut self, items: &[T]) {
        let new_len = self.length + items.len();
        if new_len > self.capacity() {
            self.reserve(new_len + 1);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                items.as_ptr(),
                self.data.as_ptr().add(self.length),
                items.len(),
            );
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
                    self.data.cast().as_ptr(),
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
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.length) }
    }
}

impl<T> std::ops::DerefMut for ChillVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.length) }
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
