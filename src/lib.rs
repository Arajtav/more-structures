#![allow(clippy::inline_always)]
#![warn(missing_docs)]

//! A crate containing a few more data structures.

pub mod overwriting_ring_buf;
pub use overwriting_ring_buf::OverwritingRingBuf;
