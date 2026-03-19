use std::marker::PhantomData;

use crate::OverwritingRingBuf;

impl<T, const L: usize> OverwritingRingBuf<T, L> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T, L> {
        <&mut Self as IntoIterator>::into_iter(self)
    }
}

pub struct IterMut<'a, T, const L: usize> {
    orb: *mut OverwritingRingBuf<T, L>,
    pos: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T, const L: usize> Iterator for IterMut<'a, T, L> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: the pointer was a valid mutable reference before.
        let orb = unsafe { &mut *self.orb };
        if self.pos >= orb.length {
            None
        } else {
            let idx = orb.read_index(self.pos);
            self.pos += 1;
            // SAFETY: the element is in 0..self.length, therefore is valid.
            Some(unsafe { orb.inner[idx].assume_init_mut() })
        }
    }
}

impl<'a, T, const L: usize> IntoIterator for &'a mut OverwritingRingBuf<T, L> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, L>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            orb: self,
            pos: 0,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_mut() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);
        for el in &mut buf {
            *el *= 4;
        }
        assert_eq!(
            buf.iter().copied().collect::<Vec<i32>>(),
            vec![4, 8, 12, 16]
        );
    }
}
