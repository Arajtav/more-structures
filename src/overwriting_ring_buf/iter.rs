use crate::OverwritingRingBuf;

impl<T, const L: usize> OverwritingRingBuf<T, L> {
    /// Returns an iterator over the elements in insertion order.
    pub fn iter(&self) -> Iter<'_, T, L> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

/// An iterator over the elements of an `OverwritingRingBuf`.
pub struct Iter<'a, T, const L: usize> {
    orb: &'a OverwritingRingBuf<T, L>,
    pos: usize,
}

impl<'a, T, const L: usize> Iterator for Iter<'a, T, L> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.orb.length {
            None
        } else {
            let idx = self.orb.read_index(self.pos);
            self.pos += 1;
            // SAFETY: the element is in 0..self.length, therefore is valid.
            Some(unsafe { self.orb.inner[idx].assume_init_ref() })
        }
    }
}

impl<'a, T, const L: usize> IntoIterator for &'a OverwritingRingBuf<T, L> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, L>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { orb: self, pos: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);

        assert_eq!(buf.iter().copied().collect::<Vec<i32>>(), vec![1, 2, 3, 4]);
    }
}
