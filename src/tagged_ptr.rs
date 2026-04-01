//! Contains tagged pointer types.

mod tagged_const_ptr;
mod tagged_mut;
mod tagged_mut_ptr;
mod tagged_ref;

pub use tagged_const_ptr::TaggedConstPtr;
pub use tagged_mut::TaggedMut;
pub use tagged_mut_ptr::TaggedMutPtr;
pub use tagged_ref::TaggedRef;
