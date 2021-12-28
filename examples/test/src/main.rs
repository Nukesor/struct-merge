mod structs;

use struct_merge::{StructMerge, StructMergeRef};

use crate::structs::*;

fn main() {
    merge();
    merge_soft();
}

/// Test the normal [StructMerge::merge] and [StructMergeRef::merge_soft] functions.
fn merge() {
    // The base struct that's going to be merged into.
    let mut base = Base::new();

    // Merge a struct with identical field types.
    let identical = Identical::new();
    base.merge_ref(&identical);
    assert_eq!(base.normal, "identical".to_string());
    assert_eq!(base.optional, Some("identical".to_string()));

    // Merge a struct with the same field types, but they're optional.
    let optional = Optional::new();
    base.merge_ref(&optional);
    assert_eq!(base.normal, "optional".to_string());
    assert_eq!(base.optional, Some("optional".to_string()));

    // Merge a struct with both, identical and optional fields.
    let mixed = Mixed::new();
    base.merge(mixed);
    assert_eq!(base.normal, "mixed".to_string());
    assert_eq!(base.optional, Some("mixed".to_string()));
}

/// Test the normal [StructMerge::merge] and [StructMergeRef::merge_soft] functions.
///
/// In soft-mode, only `Option`al fields will be merged.
/// They'll also only be merged, if they're currently `None`.
fn merge_soft() {
    // The base struct that's going to be merged into.
    let mut base = Base::new();
    // A struct with the same field types, but they're optional.
    let optional = Optional::new();
    // A struct with identical field types.
    let identical = Identical::new();
    // A struct with both, identical and optional fields.
    let mixed = Mixed::new();

    // Only optional fields will be merged in soft-mode.
    // `base.optional` will also not change, as it's already `Some`.
    base.merge_ref_soft(&identical);
    assert_eq!(base.normal, "base".to_string());
    assert_eq!(base.optional, Some("base".to_string()));

    // Reset to None, so we can observe a merge.
    base.optional = None;
    base.merge_ref_soft(&optional);
    assert_eq!(base.optional, Some("optional".to_string()));

    // The field shouldn't change, as it's currently `Some`
    base.merge_soft(mixed.clone());
    assert_eq!(base.normal, "base".to_string());
    assert_eq!(base.optional, Some("optional".to_string()));

    // Reset to None, so we can observe a merge.
    base.optional = None;
    base.merge_soft(mixed);
    assert_eq!(base.optional, Some("mixed".to_string()));
}
