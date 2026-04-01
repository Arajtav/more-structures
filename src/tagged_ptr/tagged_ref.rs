//! A pointer that stores extra data in bits that are unused due to alignment.

use std::{marker::PhantomData, ops::Deref};

/// A pointer that stores extra data.
pub struct TaggedRef<'a, T> {
    ptr: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> TaggedRef<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self {
            ptr: value as *const T as usize,
            _marker: PhantomData,
        }
    }

    #[inline]
    const fn mask(&self) -> usize {
        align_of::<T>() - 1
    }

    pub const fn max(&self) -> usize {
        self.mask()
    }

    #[inline]
    const fn clean(&self) -> usize {
        self.ptr & !self.mask()
    }

    pub fn set(&mut self, value: usize) {
        let mask = self.mask();
        if value > mask {
            panic!("value > max ({value} > {mask})");
        }

        self.ptr = self.clean() | value;
    }

    pub const fn get(&self) -> usize {
        self.ptr & self.mask()
    }
}

impl<'a, T> Deref for TaggedRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: converting back to a shared reference, PhantomDate ensures it is still borrowed.
        // self.clean() returns aligned version (original) of the pointer.
        unsafe { &*(self.clean() as *const T) }
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;

    #[test]
    fn test() {
        let mut a: [u32; 256] = std::hint::black_box(array::from_fn(|i| i as u32));
        assert_eq!(align_of_val(&a), 4);

        let f = &mut a[17];
        assert_eq!(*f, 17);

        let mut f = TaggedRef::new(f);
        assert_eq!(f.max(), 3);

        assert_eq!(*f, 17);
        assert_eq!(f.get(), 0);
        f.set(1);
        assert_eq!(*f, 17);
        assert_eq!(f.get(), 1);
    }
}
