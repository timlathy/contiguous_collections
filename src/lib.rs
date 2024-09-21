//! A library of collections backed by flat contiguous arrays.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub use array2::Array2;
pub use ordvec::{OrdVec, OrdVecKey, OrdVecKeyFst};

mod array2;
mod ordvec;
