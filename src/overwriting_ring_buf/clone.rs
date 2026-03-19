use std::mem::MaybeUninit;

use crate::OverwritingRingBuf;

impl<T: Clone, const L: usize> Clone for OverwritingRingBuf<T, L> {
    fn clone(&self) -> Self {
        let mut inner = [const { MaybeUninit::uninit() }; L];

        for (i, item) in inner.iter_mut().enumerate().take(self.length) {
            // SAFETY: the element is in 0..self.length, therefore is valid.
            item.write(unsafe { self.inner[self.read_index(i)].assume_init_ref() }.clone());
        }

        Self {
            // new index, since the elements were written from the beginning.
            wr_index: if self.length == L { 0 } else { self.length },
            length: self.length,
            inner,
        }
    }
}

// TODO: Not possible as there is Drop.
// impl<T: Copy, const L: usize> Copy for OverwritingRingBuf<T, L> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_eq() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(6);
        buf.push(8);
        buf.push(9);
        buf.push(10);
        buf.push(11);
        let buf2 = buf.clone();
        assert_eq!(buf, buf2);
        assert_eq!(
            buf.into_iter().collect::<Vec<i32>>(),
            buf2.into_iter().collect::<Vec<i32>>()
        );
    }
}
