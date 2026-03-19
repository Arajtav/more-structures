use crate::OverwritingRingBuf;

impl<T: PartialOrd, const L: usize> PartialOrd for OverwritingRingBuf<T, L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord, const L: usize> Ord for OverwritingRingBuf<T, L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}
