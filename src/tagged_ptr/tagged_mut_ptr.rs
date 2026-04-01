//! A pointer that stores extra data in bits that are unused due to alignment.

use std::marker::PhantomData;

/// A pointer that stores extra data.
pub struct TaggedMutPtr<T> {
    ptr: usize,
    _marker: PhantomData<*mut T>,
}

impl<T> TaggedMutPtr<T> {
    /// Creates a new tagged pointer from a normal one with the tag value set to 0.
    pub fn new_zero(value: *mut T) -> Self {
        Self {
            ptr: value as usize,
            _marker: PhantomData,
        }
    }

    /// Creates a new tagged pointer from a normal one and a tag.
    pub fn new(value: *mut T, tag: usize) -> Self {
        let mut new = Self::new_zero(value);
        new.set_tag(tag);
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

    /// Gets the original pointer.
    #[inline]
    #[must_use]
    pub const fn original(&self) -> *mut T {
        self.clean() as *mut T
    }

    /// Sets the value of the tag.
    ///
    /// # Panics
    ///
    /// Panics if value is higher than `self.max()`.
    pub fn set_tag(&mut self, value: usize) {
        let mask = self.max();
        assert!(value <= mask, "value > max ({value} > {mask})");

        self.ptr = self.clean() | value;
    }

    /// Gets the value of the tag.
    #[must_use]
    pub const fn get_tag(&self) -> usize {
        self.ptr & self.max()
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

        let mut f = TaggedMutPtr::new_zero(f);
        assert_eq!(f.max(), 3);

        assert_eq!(unsafe { *f.original() }, 17);
        assert_eq!(f.get_tag(), 0);
        f.set_tag(1);
        assert_eq!(unsafe { *f.original() }, 17);
        assert_eq!(f.get_tag(), 1);
    }
}
