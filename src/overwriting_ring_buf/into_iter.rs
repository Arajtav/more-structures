use crate::OverwritingRingBuf;

pub struct IntoIter<T, const L: usize> {
    orb: OverwritingRingBuf<T, L>,
    pos: usize,
}

impl<T, const L: usize> Iterator for IntoIter<T, L> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.orb.length {
            None
        } else {
            let idx = self.orb.read_index(self.pos);
            self.pos += 1;
            // SAFETY: the element is in 0..self.length, therefore is valid.
            Some(unsafe { self.orb.inner[idx].assume_init_read() })
        }
    }
}

impl<T, const L: usize> IntoIterator for OverwritingRingBuf<T, L> {
    type Item = T;
    type IntoIter = IntoIter<T, L>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { orb: self, pos: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_iter() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);

        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![1, 2, 3, 4]);
    }
}
