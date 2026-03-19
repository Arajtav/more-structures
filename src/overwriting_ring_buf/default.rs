use crate::OverwritingRingBuf;

impl<T, const L: usize> Default for OverwritingRingBuf<T, L> {
    fn default() -> Self {
        Self::new()
    }
}
