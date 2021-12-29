pub use struct_merge_codegen::*;

/// Merge another struct into `Self`.
pub trait StructMerge<Src> {
    /// Merge the given struct into `Self` whilst consuming it.
    fn merge(&mut self, src: Src);

    /// Merge the given struct into `Self` whilst consuming it.
    ///
    /// Nearly the same as `merge`, but any `Self::Option<T>` fields will only get merged if the
    /// value of the field is `None`.
    ///
    /// For example:
    /// ```ignore
    /// struct Target { a: Option<String> };
    /// struct Src { a: String };
    ///
    /// let target = Target { a: Some("test".to_string()) };
    /// let src = Src { a: "test2".to_string() };
    ///
    /// target.merge_soft(src);
    /// // Value didn't get merged as `target.a` was `Some`
    /// assert_eq!(target.a, "test".to_string());
    /// ```
    fn merge_soft(&mut self, src: Src);
}

/// Counterpart of [StructMerge].
/// This will merge `Self` into a given target.
pub trait StructMergeInto<Target: ?Sized> {
    /// Check the [StructMerge::merge] docs.
    fn merge_into(self, target: &mut Target);

    /// Check the [StructMerge::merge_soft] docs.
    fn merge_into_soft(self, target: &mut Target);
}

/// Implement the [StructMerge] trait for all types that provide [StructMergeInto] for it.
impl<Target, Src: StructMergeInto<Target>> StructMerge<Src> for Target {
    fn merge(&mut self, src: Src) {
        src.merge_into(self);
    }

    fn merge_soft(&mut self, src: Src) {
        src.merge_into_soft(self);
    }
}

/// Merge another borrowed struct into `Self`.
///
/// All fields to be merged on the borrowed struct have to implement [Clone].
pub trait StructMergeRef<Src> {
    /// Merge the given struct into `Self`.
    fn merge_ref(&mut self, src: &Src);

    /// Merge the given struct into `Self`.
    ///
    /// Nearly the same as `merge_ref`, but any `Self::Option<T>` fields will only get merged if the
    /// value of the field is `None`.
    ///
    /// For example:
    /// ```ignore
    /// struct Target { a: Option<String> };
    /// struct Src { a: String };
    ///
    /// let target = Target { a: Some("test".to_string()) };
    /// let src = Src { a: "test2".to_string() };
    ///
    /// target.merge_ref_soft(&src);
    /// // Value didn't get merged as `target.a` was `Some`
    /// assert_eq!(target.a, "test".to_string());
    /// ```
    fn merge_ref_soft(&mut self, src: &Src);
}

/// Counterpart of [StructMergeRef].
/// This will merge `&Self` into a given target.
pub trait StructMergeIntoRef<Target: ?Sized> {
    /// Check the [StructMergeRef::merge_ref] docs.
    fn merge_into_ref(&self, target: &mut Target);

    /// Check the [StructMergeRef::merge_ref_soft] docs.
    fn merge_into_ref_soft(&self, target: &mut Target);
}

/// Implement the [StructMergeRef] trait for all types that provide [StructMergeInto] for it.
impl<Target, Src: StructMergeIntoRef<Target>> StructMergeRef<Src> for Target {
    fn merge_ref(&mut self, src: &Src) {
        src.merge_into_ref(self);
    }

    fn merge_ref_soft(&mut self, src: &Src) {
        src.merge_into_ref_soft(self);
    }
}

pub mod prelude {
    pub use super::{StructMerge, StructMergeRef};
}
