use std::hash::{Hash, Hasher};

use crate::OverwritingRingBuf;

impl<T: Hash, const L: usize> Hash for OverwritingRingBuf<T, L> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for item in self {
            item.hash(state);
        }
    }
}
