use vec::ChillVec as Vec;

// One might expect this to be backed by a String, but to do so would not make this code panicless
// String is backed by a RawVec, which can panic when it expands its allocation if the allocation
// size would overflow an isize (we abort instead)
//
// We enforce UTF-8 by limiting the interface to accept &str. This is sufficient; no need to
// reimplement the enormous amount of logic in std::string::String
/// This can be thought of as an array of strings, but all stored in the same allocation.
/// Insertion into this data structure should not be assumed to be fast, though it is constant-time
/// the occasional large allocation will occur. However, this data structure should substantially
/// outperform a `Vec<String>` for operations that iterate over the collection.
/// A StrVec may have less memory overhead than a Vec<String>, as each std::string::String must
/// store 3 pointer-size ints along with its data a StrVec only stores one.
pub struct StrVec {
    data: Vec<u8>,
    indices: Vec<usize>,
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
        let mut indices = Vec::with_capacity(8);
        indices.push(0);
        StrVec {
            data: Vec::with_capacity(64),
            indices,
        }
    }

    pub fn with_capacity(bytes_cap: usize, indices_cap: usize) -> Self {
        let mut indices = Vec::with_capacity(indices_cap);
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
        let begin = *self.indices.get(index)?;
        let end = *self.indices.get(index + 1)?;
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
        assert_eq!(words.indices.get(0), Some(&0));

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
