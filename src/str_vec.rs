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
        let out = self.strvec.get(self.index);
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

impl StrVec {
    #[no_panic]
    pub fn new() -> Self {
        let mut indices = Vec::new();
        indices.push(0);
        StrVec {
            data: Vec::new(),
            indices,
        }
    }

    #[no_panic]
    pub fn get(&self, index: usize) -> Option<&str> {
        let begin = *self.indices.get(index)?;
        let end = *self.indices.get(index + 1)?;
        let bytes = self.data.get(begin..end)?;
        Some(unsafe { core::str::from_utf8_unchecked(bytes) })
    }

    #[no_panic]
    pub fn push(&mut self, item: &str) {
        if item.len() > 0 {
            self.indices.push(self.data.len() + item.len());
            self.data.extend_from_slice(item.as_bytes());
        }
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
        words.push("a");
        assert_eq!(words.len(), 1);
        words.push("ab");
        assert_eq!(words.len(), 2);
        words.push("abc");
        assert_eq!(words.len(), 3);
        assert_eq!(words.get(0), Some("a"));
        assert_eq!(words.get(1), Some("ab"));
        assert_eq!(words.get(2), Some("abc"));
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
