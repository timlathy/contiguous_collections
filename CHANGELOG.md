# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2024-09-22

### Added

* `Array2::subarray` method to extract a slice of rows and columns of the array.
* `Array2::map` method to create a new array by applying a function to each element.

## [0.1.0] - 2024-09-21

### Added

* `Array2<T>`, a fixed-size two-dimensional array of `T`s stored as a flat boxed slice in row-major order.
* `OrdVec<T, K>`, an ordered `Vec<T>` intended for fast lookup of items by key, with the key stored inside each `T` and retrieved via the key function `K`.

[unreleased]: https://github.com/timlathy/contiguous_collections/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/timlathy/contiguous_collections/releases/tag/v0.2.0
[0.1.0]: https://github.com/timlathy/contiguous_collections/releases/tag/v0.1.0
