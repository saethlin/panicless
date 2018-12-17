use vec::ChillVec as Vec;

// One might expect this to be backed by a String, but to do so would not make this code panicless
// String is backed by a RawVec, which can panic when it expands its allocation if the allocation
// size would overflow an isize (we abort instead)
pub struct StrVec {
    data: Vec<u8>,
    indices: NumberVec,
}

/// A dense data structure for storing ints that will expand the underlying allocation as required
/// to accomodate values being pushed into it
///
/// A NumberVec behaves as if it's a Vec<usize>, but internally stores the data much more densely
#[derive(Clone, Debug)]
enum NumberVec {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
}

impl NumberVec {
    pub fn push(&mut self, item: usize) {
        // No resize required, this is the common case
        if item <= self.max_value() {
            self.push_impl(item);
        } else {
            // Guess each size, starting from the smallest we could be resizing to
            if item <= u16::max_value() as usize {
                let mut new_contents: Vec<u16> = Vec::with_capacity(self.capacity());
                new_contents.push(item as u16);
                *self = NumberVec::U16(new_contents);
            } else if item <= u32::max_value() as usize {
                let mut new_contents: Vec<u32> = Vec::with_capacity(self.capacity());
                new_contents.push(item as u32);
                *self = NumberVec::U32(new_contents);
            } else {
                let mut new_contents: Vec<u64> = Vec::with_capacity(self.capacity());
                new_contents.push(item as u64);
                *self = NumberVec::U64(new_contents);
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        match self {
            NumberVec::U8(v) => v.get(index).map(|v| *v as usize),
            NumberVec::U16(v) => v.get(index).map(|v| *v as usize),
            NumberVec::U32(v) => v.get(index).map(|v| *v as usize),
            NumberVec::U64(v) => v.get(index).map(|v| *v as usize),
        }
    }

    fn max_value(&self) -> usize {
        match self {
            NumberVec::U8(_) => u8::max_value() as usize,
            NumberVec::U16(_) => u16::max_value() as usize,
            NumberVec::U32(_) => u32::max_value() as usize,
            NumberVec::U64(_) => u64::max_value() as usize,
        }
    }

    fn push_impl(&mut self, item: usize) {
        match self {
            NumberVec::U8(ref mut v) => v.push(item as u8),
            NumberVec::U16(ref mut v) => v.push(item as u16),
            NumberVec::U32(ref mut v) => v.push(item as u32),
            NumberVec::U64(ref mut v) => v.push(item as u64),
        }
    }

    fn capacity(&self) -> usize {
        match self {
            NumberVec::U8(v) => v.capacity(),
            NumberVec::U16(v) => v.capacity(),
            NumberVec::U32(v) => v.capacity(),
            NumberVec::U64(v) => v.capacity(),
        }
    }

    fn len(&self) -> usize {
        match self {
            NumberVec::U8(v) => v.len(),
            NumberVec::U16(v) => v.len(),
            NumberVec::U32(v) => v.len(),
            NumberVec::U64(v) => v.len(),
        }
    }
}

pub struct StrVecIter<'a> {
    strvec: &'a StrVec,
    index: usize,
}

impl<'a> Iterator for StrVecIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.index < self.len() {
            self.strvec.get(self.index)
        } else {
            None
        };
        self.index += 1;
        out
    }
}

impl<'a> ExactSizeIterator for StrVecIter<'a> {
    fn len(&self) -> usize {
        self.strvec.len()
    }
}

impl Default for StrVec {
    fn default() -> Self {
        Self::new()
    }
}

impl StrVec {
    pub fn new() -> Self {
        let mut indices = NumberVec::U8(Vec::with_capacity(8));
        indices.push(0);
        StrVec {
            data: Vec::with_capacity(64),
            indices,
        }
    }

    pub fn with_capacity(bytes_cap: usize, indices_cap: usize) -> Self {
        let mut indices = NumberVec::U8(Vec::with_capacity(indices_cap));
        indices.push(0);

        StrVec {
            data: Vec::with_capacity(bytes_cap),
            indices,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        let begin = self.indices.get(index)?;
        let end = self.indices.get(index + 1)?;
        self.data
            .get(begin..end)
            .map(|b| unsafe { std::str::from_utf8_unchecked(b) })
    }

    pub fn push(&mut self, item: &str) {
        self.indices.push(self.data.len() + item.len());
        self.data.extend_from_slice(item.as_bytes());
    }

    pub fn iter(&self) -> StrVecIter {
        StrVecIter {
            strvec: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.indices.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_assumptions() {
        let words = StrVec::new();
        assert_eq!(words.indices.len(), 1);
        assert_eq!(words.indices.get(0), Some(0));

        let iter = words.iter();
        assert_eq!(iter.index, 0);
    }

    #[test]
    fn push_get() {
        let mut words = StrVec::new();
        assert_eq!(words.len(), 0);
        words.push("a");
        assert_eq!(words.len(), 1);
        words.push("ab");
        assert_eq!(words.len(), 2);
        words.push("abc");
        assert_eq!(words.len(), 3);

        assert_eq!(words.get(0), Some("a"));
        assert_eq!(words.get(1), Some("ab"));
        assert_eq!(words.get(2), Some("abc"));
        assert_eq!(words.get(3), None);
    }

    #[test]
    fn iterate() {
        let mut words = StrVec::new();
        words.push("a");
        words.push("ab");
        words.push("abc");
        let mut iter = words.iter();
        assert_eq!(iter.next(), Some("a"));
        assert_eq!(iter.next(), Some("ab"));
        assert_eq!(iter.next(), Some("abc"));
        assert_eq!(iter.next(), None);
    }
}
