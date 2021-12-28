pub use struct_merge_macro::*;

/// Merge another struct into Self.
pub trait StructMerge<Src> {
    /// Merge the given struct into self while consuming it.
    fn merge(&mut self, src: Src);

    /// Merge the given struct into self while consuming it.
    ///
    /// Nearly the same as `merge`, but any `Self::Option<T>` fields will only get merged, if the
    /// value of the field is `None`.
    ///
    /// For example:
    /// ```ignore
    /// struct Dest { a: Option<String> };
    /// struct Src { a: String };
    ///
    /// let dest = Dest { a: Some("test".to_string()) };
    /// let src = Src { a: "test2".to_string() };
    ///
    /// dest.merge_soft(src);
    /// // Value didn't get merged, because `dest.a` was `Some`
    /// assert_eq!(dest.a, "test".to_string());
    /// ```
    fn merge_soft(&mut self, src: Src);
}

/// Merge another borrowed struct into Self.
///
/// All fields to be merged have to implement [Clone].
pub trait StructMergeRef<Src> {
    /// Merge the given struct into self.
    fn merge_ref(&mut self, src: &Src);

    /// Merge the given struct into self.
    ///
    /// Nearly the same as `merge_ref`, but any `Self::Option<T>` fields will only get merged, if the
    /// value of the field is `None`.
    ///
    /// For example:
    /// ```ignore
    /// struct Dest { a: Option<String> };
    /// struct Src { a: String };
    ///
    /// let dest = Dest { a: Some("test".to_string()) };
    /// let src = Src { a: "test2".to_string() };
    ///
    /// dest.merge_ref_soft(&src);
    /// // Value didn't get merged, because `dest.a` was `Some`
    /// assert_eq!(dest.a, "test".to_string());
    /// ```
    fn merge_ref_soft(&mut self, src: &Src);
}
