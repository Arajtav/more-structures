use std::fmt::{Debug, Formatter, Result};

use crate::OverwritingRingBuf;

impl<T: Debug, const L: usize> Debug for OverwritingRingBuf<T, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut list = f.debug_list();
        list.entries(self.iter());
        list.finish()
    }
}
