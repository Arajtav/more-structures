use crate::OverwritingRingBuf;

impl<T: PartialEq, const L: usize> PartialEq for OverwritingRingBuf<T, L> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Eq, const L: usize> Eq for OverwritingRingBuf<T, L> {}
