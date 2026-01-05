use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

// L > 0
pub struct OverwritingRingBuf<T, const L: usize> {
    /// Index of the next element to be written to. always in [0; L).
    wr_index: usize,
    /// The number of valid elements. always in [0; L].
    length: usize,
    inner: [MaybeUninit<T>; L],
}

impl<T, const L: usize> Drop for OverwritingRingBuf<T, L> {
    fn drop(&mut self) {
        for i in 0..self.length {
            // SAFETY: the element is in 0..self.length, therefore is valid.
            unsafe { self.inner[self.read_index(i)].assume_init_drop() };
        }
    }
}

impl<T, const L: usize> Default for OverwritingRingBuf<T, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const L: usize> OverwritingRingBuf<T, L> {
    #[must_use]
    pub fn new() -> Self {
        const { assert!(L > 0) }
        Self {
            wr_index: 0,
            length: 0,
            inner: [const { MaybeUninit::uninit() }; L],
        }
    }

    #[inline(always)]
    fn write_index(&self, i: usize) -> usize {
        self.wr_index.wrapping_add(i) % L
    }

    /// Converts logical to physical indexes in self.inner.
    /// Guaranteed to return valid indexes for `i < self.length`.
    /// 0 is the oldest element. self.length-1 is the newest.
    #[inline(always)]
    fn read_index(&self, i: usize) -> usize {
        debug_assert!(i < self.length);
        self.wr_index.wrapping_sub(self.length).wrapping_add(i) % L
    }

    pub fn push(&mut self, new: T) -> Option<T> {
        if self.is_full() {
            // SAFETY: length == L => all elements are initialized.
            let old = unsafe { self.inner[self.wr_index].assume_init_read() };
            self.inner[self.wr_index].write(new);
            self.wr_index = self.write_index(1);
            Some(old)
        } else {
            self.inner[self.wr_index].write(new);
            self.wr_index = self.write_index(1);
            self.length += 1;
            None
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.length {
            // SAFETY: 0..self.length covers all the elements.
            unsafe {
                self.inner[self.read_index(i)].assume_init_drop();
            }
        }
        self.length = 0;
        self.wr_index = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        L
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.length == L
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let idx = self.read_index(0);
        self.length -= 1;

        // SAFETY: self.length > 0; read_index(0) is guaranteed to return the oldest element.
        Some(unsafe { self.inner[idx].assume_init_read() })
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut f: F) {
        if self.is_empty() {
            return;
        }

        #[cfg(debug_assertions)]
        let debug_tmp = self.read_index(0);

        let mut j = 0;
        for i in 0..self.length {
            // SAFETY: the element is in 0..self.length, therefore is valid.
            let current = unsafe { self.inner[self.read_index(i)].assume_init_read() };

            if f(&current) {
                // `current` is a copy, we either write it into a different place or forget it if it
                // is in the right place.
                if i == j {
                    std::mem::forget(current);
                } else {
                    self.inner[self.read_index(j)].write(current);
                }
                j += 1;
            } else {
                drop(current);
            }
        }

        // wr_index must be read_index(0) + length
        self.wr_index = (self.wr_index.wrapping_sub(self.length).wrapping_add(j)) % L;
        self.length = j;

        // make sure after those changes read_index still points to the same element.
        #[cfg(debug_assertions)]
        if !self.is_empty() {
            debug_assert_eq!(debug_tmp, self.read_index(0));
        }
    }

    // a bit modified copy of retain
    pub fn retain_mut<F: FnMut(&mut T) -> bool>(&mut self, mut f: F) {
        if self.is_empty() {
            return;
        }

        #[cfg(debug_assertions)]
        let debug_tmp = self.read_index(0);

        let mut j = 0;
        for i in 0..self.length {
            // SAFETY: the element is in 0..self.length, therefore is valid.
            let mut current = unsafe { self.inner[self.read_index(i)].assume_init_read() };

            if f(&mut current) {
                self.inner[self.read_index(j)].write(current);
                j += 1;
            } else {
                drop(current);
            }
        }

        // wr_index must be read_index(0) + length
        self.wr_index = (self.wr_index.wrapping_sub(self.length).wrapping_add(j)) % L;
        self.length = j;

        // make sure after those changes read_index still points to the same element.
        #[cfg(debug_assertions)]
        if !self.is_empty() {
            debug_assert_eq!(debug_tmp, self.read_index(0));
        }
    }

    pub fn iter(&self) -> OverwritingRingBufferIter<'_, T, L> {
        OverwritingRingBufferIter { orb: self, pos: 0 }
    }

    pub fn iter_mut(&mut self) -> OverwritingRingBufferIterMut<'_, T, L> {
        <&mut Self as IntoIterator>::into_iter(self)
    }
}

pub struct OverwritingRingBufferIter<'a, T, const L: usize> {
    orb: &'a OverwritingRingBuf<T, L>,
    pos: usize,
}

impl<'a, T, const L: usize> Iterator for OverwritingRingBufferIter<'a, T, L> {
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
    type IntoIter = OverwritingRingBufferIter<'a, T, L>;

    fn into_iter(self) -> Self::IntoIter {
        OverwritingRingBufferIter { orb: self, pos: 0 }
    }
}

pub struct OverwritingRingBufferIntoIter<T, const L: usize> {
    orb: OverwritingRingBuf<T, L>,
    pos: usize,
}

impl<T, const L: usize> Iterator for OverwritingRingBufferIntoIter<T, L> {
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
    type IntoIter = OverwritingRingBufferIntoIter<T, L>;

    fn into_iter(self) -> Self::IntoIter {
        OverwritingRingBufferIntoIter { orb: self, pos: 0 }
    }
}

pub struct OverwritingRingBufferIterMut<'a, T, const L: usize> {
    orb: *mut OverwritingRingBuf<T, L>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T, const L: usize> Iterator for OverwritingRingBufferIterMut<'a, T, L> {
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
    type IntoIter = OverwritingRingBufferIterMut<'a, T, L>;

    fn into_iter(self) -> Self::IntoIter {
        OverwritingRingBufferIterMut {
            orb: self,
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

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
    use super::OverwritingRingBuf;

    #[test]
    fn basic_info() {
        let buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 4);
        assert!(!buf.is_full());
    }

    #[test]
    fn push_overwrite() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();

        assert_eq!(buf.push(1), None);
        assert_eq!(buf.push(2), None);
        assert_eq!(buf.push(3), None);
        assert_eq!(buf.push(4), None);
        assert_eq!(buf.push(5), Some(1));
        assert_eq!(buf.push(6), Some(2));
    }

    #[test]
    fn pop() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);
        assert_eq!(buf.pop(), Some(1));
        buf.push(5);
        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![2, 3, 4, 5]);
    }

    #[test]
    fn clear() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(0);
        assert!(buf.len() == 1);
        buf.clear();
        assert!(buf.is_empty());
        buf.push(3);
        buf.push(2);
        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![3, 2]);
    }

    #[test]
    fn retain() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);

        buf.retain(|e| e % 2 == 0); // remove odd elements
        assert_eq!(buf.iter().copied().collect::<Vec<i32>>(), vec![2, 4]);

        buf.push(8);
        buf.push(9);
        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![2, 4, 8, 9]);
    }

    #[test]
    fn iters() {
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

    #[test]
    fn retain_mut() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);

        // remove odd elements, mul even by 4
        buf.retain_mut(|e| {
            *e *= 4;
            *e % 8 == 0
        });
        assert_eq!(buf.iter().copied().collect::<Vec<i32>>(), vec![8, 16]);

        buf.push(8);
        buf.push(9);
        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![8, 16, 8, 9]);
    }

    #[test]
    fn index() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        buf.push(6);
        buf.push(7);
        buf.push(8);
        buf.push(9);

        buf[2] = 10;

        assert_eq!(buf[0], 6);
        assert_eq!(buf[1], 7);
        assert_eq!(buf[2], 10);
        assert_eq!(buf[3], 9);
    }

    #[test]
    #[should_panic(expected = "assertion failed: index < self.length")]
    fn out_of_bounds_index() {
        let buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        let _ = buf[0];
    }

    #[test]
    fn zst() {
        assert_eq!(std::mem::size_of::<()>(), 0);
        let mut buf: OverwritingRingBuf<(), 4> = OverwritingRingBuf::new();
        assert_eq!(buf.capacity(), 4);
        assert!(buf.is_empty());

        buf.push(());
        assert_eq!(buf.len(), 1);

        buf.push(());
        buf.push(());
        buf.push(());
        assert_eq!(buf.len(), 4);

        assert_eq!(buf.pop(), Some(()));
        assert_eq!(buf.into_iter().collect::<Vec<()>>(), vec![(), (), ()]);
    }
}
