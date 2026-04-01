use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// A pointer that stores extra data.
pub struct TaggedMut<'a, T> {
    ptr: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> TaggedMut<'a, T> {
    /// Creates a new tagged pointer from a normal one with the tag value set to 0.
    pub fn new_zero(value: &'a mut T) -> Self {
        Self {
            ptr: std::ptr::from_ref::<T>(value) as usize,
            _marker: PhantomData,
        }
    }

    /// Creates a new tagged pointer from a normal one and a tag.
    pub fn new(value: &'a mut T, tag: usize) -> Self {
        let mut new = Self::new_zero(value);
        new.set(tag);
        new
    }

    /// The max value that can be written into the tag.
    #[inline]
    #[must_use]
    pub const fn max(&self) -> usize {
        align_of::<T>() - 1
    }

    #[inline]
    const fn clean(&self) -> usize {
        self.ptr & !self.max()
    }

    /// Sets the value of the tag.
    ///
    /// # Panics
    ///
    /// Panics if value is higher than `self.max()`.
    pub fn set(&mut self, value: usize) {
        let mask = self.max();
        assert!(value <= mask, "value > max ({value} > {mask})");

        self.ptr = self.clean() | value;
    }

    /// Gets the value of the tag.
    #[must_use]
    pub const fn get(&self) -> usize {
        self.ptr & self.max()
    }
}

impl<T> Deref for TaggedMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: converting back to a shared reference, PhantomData ensures it is still borrowed.
        // self.clean() returns aligned (original) version of the pointer.
        unsafe { &*(self.clean() as *const T) }
    }
}

impl<T> DerefMut for TaggedMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: converting back to a shared reference, PhantomData ensures it is still borrowed.
        // self.clean() returns aligned (original) version of the pointer.
        unsafe { &mut *(self.clean() as *mut T) }
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation)]
mod tests {
    use std::array;

    use super::*;

    #[test]
    fn test() {
        let mut a: [u32; 256] = std::hint::black_box(array::from_fn(|i| i as u32));
        assert_eq!(align_of_val(&a), 4);

        let f = &mut a[17];
        assert_eq!(*f, 17);

        let mut f = TaggedMut::new_zero(f);
        assert_eq!(f.max(), 3);

        assert_eq!(*f, 17);
        *f = 20;
        assert_eq!(*f, 20);
        assert_eq!(f.get(), 0);
        *f = 18;
        f.set(1);
        assert_eq!(*f, 18);
        assert_eq!(f.get(), 1);
    }
}
