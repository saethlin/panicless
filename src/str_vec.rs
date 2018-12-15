use vec::ChillVec as Vec;

// One might expect this to be backed by a String, but to do so would not make this code panicless
// String is backed by a RawVec, which can panic when it expands its allocation if the allocation
// size would overflow an isize
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

    pub fn shrink_to_fit(&mut self) {
        self.indices.shrink_to_fit();
        self.data.shrink_to_fit();
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
