# contiguous_collections

[![Crates.io](https://img.shields.io/crates/v/contiguous_collections.svg)](https://crates.io/crates/contiguous_collections)
[![Documentation](https://docs.rs/contiguous_collections/badge.svg)](https://docs.rs/contiguous_collections)

`contiguous_collections` is a small Rust library of collections backed by flat contiguous arrays:
* [`Array2<T>`](https://docs.rs/contiguous_collections/latest/contiguous_collections/struct.Array2.html), a fixed-size two-dimensional array of `T`s stored as a flat boxed slice in row-major order.
* [`OrdVec<T, K>`](https://docs.rs/contiguous_collections/latest/contiguous_collections/struct.OrdVec.html), an ordered `Vec<T>` intended for fast lookup of items by key, with the key stored inside each `T` and retrieved via the key function `K`.
