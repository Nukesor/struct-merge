# Struct-Merge

[![GitHub Actions Workflow](https://github.com/nukesor/struct-merge/workflows/Test%20build/badge.svg)](https://github.com/Nukesor/struct-merge/actions)
[![Crates.io](https://img.shields.io/crates/v/struct-merge)](https://crates.io/crates/struct-merge)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/github/downloads/nukesor/struct-merge/total.svg)](https://github.com/nukesor/struct-merge/releases)


## !!! Deprecated !!!

This project is superseded by [inter-struct](https://github.com/Nukesor/inter-struct).

Consider switching to that library, as this library won't get any updates.

## !!! Deprecated !!!

Generate code for merging various structs.

This crate provides two proc-macros to generate two very similar traits.

```rust,ignore
/// Merge another struct into Self whilst consuming it.
/// 
/// The other trait is named `StructMergeRef` and merges other structs by reference.
pub trait StructMerge<Src> {
    /// Merge the given struct into self.
    fn merge(&mut self, src: Src);

    /// Nearly the same as `merge`, but any `Self::Option<T>` fields will only get merged, if the
    /// value of the field is `None`.
    fn merge_soft(&mut self, src: Src);
}
```

Please read the **known caveats** section before using this crate!


## Example

This example shows a simple usage of the `struct_merge` macro.

`structs.rs`: The structs that will be used in this example.
```rust,ignore
use struct_merge::struct_merge;

/// The target struct we'll merge into.
pub struct Target {
    pub normal: String,
    pub optional: Option<String>,
    /// This field won't be touched as the macro cannot find a
    /// respective `ignored` field in the `Mixed` struct.
    pub ignored: Option<String>,
}

/// A struct with both an identical and an optional field type.
/// Note that the path to `Target` must always a fully qualifying path.
#[struct_merge(crate::structs::Target)]
pub struct Mixed {
    pub normal: String,
    pub optional: Option<Option<String>>,
}
```

`lib.rs`
```rust,ignore
use struct_merge::prelude::*;

mod structs;
use structs::{Target, Mixed};

fn main() {
    let mut target = Target {
        normal: "target".to_string(),
        optional: Some("target".to_string()),
        ignored: "target".to_string(),
    };

    let mixed = Mixed {
        /// Has the same type as Target::normal
        normal: "mixed".to_string(),
        /// Wraps Target::optional in an Option
        optional: Some(Some("mixed".to_string())),
    };

    // Merge the `Mixed` struct into target.
    target.merge(mixed);
    // You can also call this:
    // mixed.merge_into(target);
    assert_eq!(target.normal, "mixed".to_string());
    assert_eq!(target.optional, Some("mixed".to_string()));
    assert_eq!(target.ignored, "target".to_string());
}
```


## Merge Behavior

The following will explain the merge behavior of a single field on the target struct.
The name of the target field is `test`.

### `merge` and `merge_ref`

#### Same Type

```rust
struct Src {
    test: T
}
struct Target {
    test: T
}
```

This will simply merge `src.test` into `target.test`: \
`target.test = src.test`

#### Target is Optional

```rust
struct Src {
    test: T
}
struct Target {
    test: Option<T>
}
```

This will wrap `src.test` into an `Option` and merge it into `target.test`: \
`target.test = Some(src.test);`

#### Source is Optional

```rust
struct Src {
    test: Option<T>
}
struct Target {
    test: T
}
```

This will only merge `src.test` into `target.test` if `src.test` is `Some`: \
```rust
if let Some(value) = src.test {
    target.test = value;
}
```

### `merge_soft` and `merge_ref_soft`

#### `target.test` is not Optional

As long as a target field is not optional it won't be touched!

#### Target is Optional

```rust
struct Src {
    test: T
}
struct Target {
    test: Option<T>
}
```

This will wrap `src.test` into an `Option` and merge it into `target.test` but only if `target.test` is `None`: \
```rust
if target.test.is_none() {
    target.test = Some(src.test);
}
```

#### Both are Optional

```rust
struct Src {
    test: Option<T>
}
struct Target {
    test: Option<T>
}
```

This will only merge `src.test` into `target.test` if `target.test` is `None`: \
```rust
if target.test.is_none() {
    target.test = src.test;
}
```


## Known caveats

### Module/Type Resolution

There is no easy way to do path or type resolution during the procedural macro stage. \
For this reason, this crate might not work with your project's structure.

However, as we're creating safe and valid Rust code the compiler will thrown an error if any problems arise.

When using the normal `merge` functions, the worst thing that might happen is that this crate won't compile.

**However**, when using obscured types such as type aliases, the `merge_soft` functions won't detect `Option`s properly and might merge values even though they're already `Some`!

#### Not yet solved problems

These are problems that can probably be solved but they're non-trivial.

- [ ] Struct located at root of crate. E.g. `lib.rs`.
- [ ] Struct is located in integration tests.
- [ ] Struct in (potentially nested or alternating) `mod {}` block in file.
- [ ] The source root dir isn't `src`.
      We would have to check the environment and possibly parse the `Cargo.toml`.
- [ ] Different generic aliases that use different tokens but have the same type.
        E.g.`Box<dyn T>` and `Box<dyn S>` but both `S` and `T` have the `Clone` trait bound.
- [ ] Non-public structs. I.e. structs that aren't fully internally visible.
    This will lead to an compiler-error but isn't cought while running this macro.
    This might be infeasible?

#### Unsolvable problems

These are problems that are either impossible to solve or very infeasible.
For instance, something infeasible would be to parse all files and to do a full type resolution of a given crate.
That would be a job for the compiler in later stages.

- Structs that are altered or generated by other macros.
- Type aliases. E.g. `type test = Option<String>` won't be detected as an Option.
    The current check for `Option` fields is a literal check for the `Option` token.
