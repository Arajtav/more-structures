use std::ops::{Index, IndexMut};

use crate::OverwritingRingBuf;

impl<T, const L: usize> Index<usize> for OverwritingRingBuf<T, L> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length);

        // SAFETY: the element is in 0..self.length, therefore is valid.
        unsafe { self.inner[self.read_index(index)].assume_init_ref() }
    }
}

impl<T, const L: usize> IndexMut<usize> for OverwritingRingBuf<T, L> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.length);

        // SAFETY: the element is in 0..self.length, therefore is valid.
        unsafe { self.inner[self.read_index(index)].assume_init_mut() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(6);
        buf.push(7);
        buf.push(8);
        buf.push(9);

        buf[2] = 10;

        // so it is not mut
        let buf = buf;

        assert_eq!(buf[0], 6);
        assert_eq!(buf[1], 7);
        assert_eq!(buf[2], 10);
        assert_eq!(buf[3], 9);
    }

    #[test]
    #[should_panic(expected = "assertion failed: index < self.length")]
    fn test_out_of_bounds_index() {
        let buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        let _ = buf[0];
    }

    #[test]
    #[should_panic(expected = "assertion failed: index < self.length")]
    fn test_out_of_bounds_index_mut() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf[0] = 1;
    }
}
