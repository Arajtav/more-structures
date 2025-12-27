use std::mem::MaybeUninit;

pub struct OverwritingRingBuf<T, const L: usize> {
    // index of the next element to write to.
    // always in [0; L)
    wr_index: usize,
    // always in [0; L]
    length: usize,
    inner: [MaybeUninit<T>; L],
}

impl<T, const L: usize> Default for OverwritingRingBuf<T, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const L: usize> OverwritingRingBuf<T, L> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            wr_index: 0,
            length: 0,
            // MaybeUninit does not need to be initialized i guess?
            inner: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    #[inline(always)]
    fn write_index(&self, i: usize) -> usize {
        self.wr_index.wrapping_add(i) % L
    }

    #[inline(always)]
    fn read_index(&self, i: usize) -> usize {
        self.wr_index.wrapping_sub(self.length).wrapping_add(i) % L
    }

    pub fn push(&mut self, new: T) -> Option<T> {
        if self.is_full() {
            // written to before
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
            let idx = self.read_index(i);
            // written to before
            unsafe {
                self.inner[idx].assume_init_drop();
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

        // written to before
        let old = unsafe { self.inner[self.read_index(0)].assume_init_read() };
        self.length -= 1;

        Some(old)
    }

    pub fn retain<F: FnMut(&T) -> bool>(&mut self, mut f: F) {
        if self.is_empty() {
            return;
        }

        let mut new_len = 0;

        for i in 0..self.length {
            let idx = self.read_index(i);
            // written to before
            let current = unsafe { self.inner[idx].assume_init_read() };

            if f(&current) {
                if new_len != i {
                    let target_idx = self.read_index(new_len);
                    self.inner[target_idx].write(current);
                }
                new_len += 1;
            } else {
                // the element is a copy. the original is kept where it was.
                std::mem::forget(current);
            }
        }

        self.length = new_len;
        self.wr_index = self.write_index(new_len);
    }

    pub fn iter(&self) -> OverwritingRingBufferIter<'_, T, L> {
        OverwritingRingBufferIter { orb: self, pos: 0 }
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
            // between [0, len)
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
            // between [0, len)
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

#[cfg(test)]
mod tests {
    use super::OverwritingRingBuf;

    #[test]
    fn basic_info() {
        let buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        assert!(buf.is_empty());
        assert!(buf.capacity() == 4);
        assert!(buf.is_empty());
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
        buf.retain(|e| e % 2 == 0);
        assert!(buf.len() == 2);
        buf.push(8);
        buf.push(9);
        assert_eq!(buf.into_iter().collect::<Vec<i32>>(), vec![2, 4, 8, 9]);
    }
}
