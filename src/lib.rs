#![allow(clippy::inline_always)]
#![warn(missing_docs)]
#![warn(clippy::missing_const_for_fn)]

//! A crate containing a few more data structures.

#[cfg(feature = "overwriting-ring-buffer")]
pub mod overwriting_ring_buf;
#[cfg(feature = "overwriting-ring-buffer")]
pub use overwriting_ring_buf::OverwritingRingBuf;
