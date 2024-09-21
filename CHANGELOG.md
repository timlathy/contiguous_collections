# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-09-21

### Added

- `Array2<T>`, a fixed-size two-dimensional array of `T`s stored as a flat boxed slice in row-major order.
* `OrdVec<T, K>`, an ordered `Vec<T>` intended for fast lookup of items by key, with the key stored inside each `T` and retrieved via the key function `K`.

[unreleased]: https://github.com/timlathy/contiguous_collections/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/timlathy/contiguous_collections/releases/tag/v0.1.0
