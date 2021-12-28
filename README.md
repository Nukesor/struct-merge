# Struct-Merge

[![GitHub Actions Workflow](https://github.com/nukesor/struct-merge/workflows/Test%20build/badge.svg)](https://github.com/Nukesor/struct-merge/actions)
[![Crates.io](https://img.shields.io/crates/v/struct-merge)](https://crates.io/crates/struct-merge)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/github/downloads/nukesor/struct-merge/total.svg)](https://github.com/nukesor/struct-merge/releases)

Generate code for merging various structs.

This crate provides two proc-macros to generate two different, but very similary traits:

```rust,ignore
/// Merge another struct into Self whilst consuming it.
pub trait StructMerge<Src> {
    /// Merge the given struct into self.
    fn merge(&mut self, src: Src);

    /// Nearly the same as `merge`, but any `Self::Option<T>` fields will only get merged, if the
    /// value of the field is `None`.
    fn merge_soft(&mut self, src: Src);
}

/// Merge another borrowed struct into Self.
/// All fields to be merged have to implement [Clone].
pub trait StructMergeRef<Src> {
    /// The same as StructMerge::merge.
    fn merge_ref(&mut self, src: &Src);

    /// Same behavior as StructMerge::merge_soft.
    fn merge_ref_soft(&mut self, src: &Src);
}
```

Please read the **known caveats** section before using this crate!

## Example

This example shows a simple usage of the `struct_merge` macro.

`structs.rs`
```rust,ignore
/// This is the definition of the structs that will be used.
use struct_merge::struct_merge;

/// The base struct we'll merge into.
pub struct Base {
    pub normal: String,
    pub optional: Option<String>,
    /// This field won't be touched, as the macro cannot find a
    /// respective `ignored` field in the `Mixed` struct.
    pub ignored: Option<String>,
}

/// A struct with both an identical and an optional field type.
/// Note that the path to `Base` must always a fully qualifying path.
#[struct_merge(crate::structs::Base)]
pub struct Mixed {
    pub normal: String,
    pub optional: Option<Option<String>>,
}
```

`lib.rs`
```rust,ignore
use struct_merge::StructMerge;
mod structs;
use structs::{Base, Mixed};

/// Test the normal [StructMerge::merge] and [StructMergeRef::merge_soft] functions.
fn main() {
    // The base struct that's going to be merged into.
    let mut base = Base {
        normal: "base".to_string(),
        optional: Some("base".to_string()),
        ignored: "base".to_string(),
    };

    // Merge the `Mixed` struct into base.
    let mixed = Mixed {
        normal: "mixed".to_string(),
        optional: Some(Some("mixed".to_string())),
    };

    base.merge(mixed);
    assert_eq!(base.normal, "mixed".to_string());
    assert_eq!(base.optional, Some("mixed".to_string()));
    assert_eq!(base.ignored, "base".to_string());
}
```

## Known caveats

### Module/Type Resolution

There is no easy way to do path or type resolution during the procedural macro stage, as everything we have are some tokens. \
That's why this crate might not work with your project's structure.
However, as we're creating safe and valid Rust code, the compiler will thrown an error if any problems arise.

When using the normal `merge` functions, the worst thing that might happen is that this crate won't compile.

**However**, when using obscured types such as type aliases, the `merge_soft` functions won't detect `Option`s properly and merge values even though they might already be `Some`!

#### Not yet solved problems

These are problems that can probably be solved, but they're non-trivial.

- [ ] Struct located at root of crate. E.g. `lib.rs`.
- [ ] Struct is located in integration tests.
- [ ] Struct in (potentially nested or alternating) `mod {}` block in file.
- [ ] The source root dir isn't `src`.
      We would have to check the environment and possibly parse the `Cargo.toml`.
- [ ] Different generic aliases that use different tokens but have the same type.
        E.g.`Box<dyn T>` and `Box<dyn S>`, but both `S` and `T` have the `Clone` trait bound.
- [ ] Non-public structs. I.e. structs that aren't fully internally visible.
    This will lead to an compiler-error, but isn't cought while running this macro.
    This might be infeasible?

#### Unsolvable problems

These are problems that are either impossible to solve or very infeasible.
For instance, something infeasible would be to parse all files and to do a full type resolution of a given crate.
That would be a job for the compiler in later stages.

- Structs that are altered or generated by other macros.
- Type aliases. E.g. `type test = Option<String>` won't be detected as an Option.
    The current check for `Option` fields is a literal check for the `Option` token.
