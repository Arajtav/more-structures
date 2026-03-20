//! A ring buffer which overwrites old values if there is no more capacity left.

mod clone;
mod debug;
mod default;
mod eq;
mod hash;
mod index;
mod into_iter;
mod iter;
mod iter_mut;
mod ord;

pub use into_iter::IntoIter;
pub use iter::Iter;
pub use iter_mut::IterMut;

use std::mem::MaybeUninit;

/// A ring buffer which overwrites old values on insert when there is no capacity left.
/// Capacity/length has to be a compile time constant greater than 0.
pub struct OverwritingRingBuf<T, const L: usize> {
    /// Index of the next element to be written to. always in [0; L).
    wr_index: usize,
    /// The number of valid elements. always in [0; L].
    length: usize,
    /// Inner buffer.
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

impl<T, const L: usize> OverwritingRingBuf<T, L> {
    /// Creates a new empty `OverwritingRingBuf`.
    #[must_use]
    pub const fn new() -> Self {
        const { assert!(L > 0) }
        Self {
            wr_index: 0,
            length: 0,
            inner: [const { MaybeUninit::uninit() }; L],
        }
    }

    #[inline(always)]
    const fn write_index(&self, i: usize) -> usize {
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

    /// Appends a value or overwrites the oldest one.
    pub const fn push(&mut self, new: T) -> Option<T> {
        if self.is_full() {
            // SAFETY: length == L => all values are initialized.
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

    /// Removes all values.
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

    /// Returns the number of elements.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Returns the total number of elements the buffer can hold without overwriting.
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        L
    }

    /// Returns true if the ring buffer is empty.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns true if the buffer will overwrite on next push.
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.length == L
    }

    /// Removes the last element and returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let idx = self.read_index(0);
        self.length -= 1;

        // SAFETY: self.length > 0; read_index(0) is guaranteed to return the oldest element.
        Some(unsafe { self.inner[idx].assume_init_read() })
    }

    /// Retains only the elements specified by the predicate. All elements are visited exactly once
    /// in original order, and the original order is preserved for retained elements.
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

    /// Retains only the elements specified by the predicate. All elements are visited exactly once
    /// in original order, and the original order is preserved for retained elements.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_info() {
        let buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 4);
        assert!(!buf.is_full());
    }

    #[test]
    fn test_push_overwrite() {
        let mut buf: OverwritingRingBuf<i32, 4> = OverwritingRingBuf::new();

        assert_eq!(buf.push(1), None);
        assert_eq!(buf.push(2), None);
        assert_eq!(buf.push(3), None);
        assert_eq!(buf.push(4), None);
        assert_eq!(buf.push(5), Some(1));
        assert_eq!(buf.push(6), Some(2));
    }

    #[test]
    fn test_pop() {
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
    fn test_clear() {
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
    fn test_retain() {
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
    fn test_retain_mut() {
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
    fn test_zst() {
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
