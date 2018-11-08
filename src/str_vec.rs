use vec::ChillVec as Vec;

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
    #[no_panic]
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.index < self.len() {
            Some(self.strvec.get(Key(self.index)))
        } else {
            None
        };
        self.index += 1;
        out
    }
}

impl<'a> ExactSizeIterator for StrVecIter<'a> {
    #[no_panic]
    fn len(&self) -> usize {
        self.strvec.len()
    }
}

#[derive(Clone, Copy)]
pub struct Key(usize);

impl From<Key> for usize {
    fn from(k: Key) -> usize {
        k.0
    }
}

impl StrVec {
    #[no_panic]
    pub fn new() -> Self {
        let mut indices = Vec::with_capacity(8);
        indices.push(0);
        StrVec {
            data: Vec::with_capacity(64),
            indices,
        }
    }

    #[no_panic]
    pub fn get(&self, key: Key) -> &str {
        let index = key.0;
        unsafe {
            let begin = *self.indices.get_unchecked(index);
            let end = *self.indices.get_unchecked(index + 1);
            let bytes = self.data.get_unchecked(begin..end);
            core::str::from_utf8_unchecked(bytes)
        }
    }

    #[no_panic]
    pub fn push(&mut self, item: &str) -> Key {
        self.indices.push(self.data.len() + item.len());
        self.data.extend_from_slice(item.as_bytes());
        Key(self.indices.len() - 2)
    }

    #[no_panic]
    pub fn iter<'a>(&'a self) -> StrVecIter<'a> {
        StrVecIter {
            strvec: self,
            index: 0,
        }
    }

    #[no_panic]
    pub fn len(&self) -> usize {
        self.indices.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        let mut words = StrVec::new();
        assert_eq!(words.len(), 0);
        let first = words.push("a");
        assert_eq!(words.len(), 1);
        let second = words.push("ab");
        assert_eq!(words.len(), 2);
        let third = words.push("abc");
        assert_eq!(words.len(), 3);

        assert_eq!(words.get(first), "a");
        assert_eq!(words.get(second), "ab");
        assert_eq!(words.get(third), "abc");
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
    }
}
