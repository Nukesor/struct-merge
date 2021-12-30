# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 30-12-2021

This is the first MVP release of the `struct_merge` library.

It's purpose is to generate code for structs with different merge strategies.

### Added

Traits:

- `StructMerge` trait which implements functions to merge a given struct into `Self`.
- `StructMergeInto` trait.
    The counterpart of `StructMerge` which merges `Self` into a target struct.
    `StructMerge` is automatically implemented.
- `StructMergeRef` trait which implements functions to merge a reference of given struct into `Self`.
    The fields to be merged then need to implement `Clone`.
- `StructMergeIntoRef` trait.
    The counterpart of `StructMergeRef`, which merges `&Self` into a target struct.
    `StructMergeRef` is automatically implemented.

Macros:

- `struct_merge` macro for generating the `StructMergeInto` and thereby the respective `StructMerge` implementations.
- `struct_merge_ref` macro for generating the `StructMergeInto` Refand thereby the respective `StructMergeRef` implementations.
