//! A library of collections backed by flat contiguous arrays.

pub use array2::Array2;
pub use ordvec::{OrdVec, OrdVecKey, OrdVecKeyFst};

mod array2;
mod ordvec;
